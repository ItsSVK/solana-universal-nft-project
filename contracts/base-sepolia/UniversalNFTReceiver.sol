// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "@openzeppelin/contracts/token/ERC20/utils/SafeERC20.sol";
import {RevertContext} from "@zetachain/protocol-contracts/contracts/Revert.sol";
import "@zetachain/protocol-contracts/contracts/evm/GatewayEVM.sol";
import "../../shared/CrossChainMessage.sol";

/**
 * @title UniversalNFTReceiver
 * @notice EVM-compatible NFT contract for Base Sepolia that receives cross-chain NFTs
 * @dev Implements ERC721 with cross-chain receiving functionality via ZetaChain Gateway
 */
contract UniversalNFTReceiver is 
    ERC721, 
    ERC721URIStorage, 
    Ownable, 
    ReentrancyGuard, 
    Pausable 
{
    using SafeERC20 for IERC20;
    using CrossChainMessage for CrossChainMessage.NFTTransferMessage;

    GatewayEVM public immutable gateway;
    
    // Chain IDs
    uint32 public constant SOLANA_DEVNET = 900;
    uint32 public constant ZETACHAIN_TESTNET = 7001;
    uint32 public constant BASE_SEPOLIA = 84532;

    // Message validation
    uint256 public constant MAX_MESSAGE_AGE = 24 hours;
    uint256 public constant MAX_METADATA_URI_LENGTH = 500;

    // ZetaChain Universal NFT contract address
    address public zetaChainUniversalNFT;

    // State variables
    uint256 private _tokenIdCounter;
    mapping(uint256 => uint256) public tokenNonces; // tokenId => nonce for outgoing transfers
    mapping(bytes32 => bool) public processedMessages; // messageId => processed status
    mapping(uint256 => CrossChainMessage.NFTTransferMessage) public tokenOrigins; // tokenId => original mint info
    mapping(address => uint256) public userNonces; // user => nonce for message uniqueness

    // Events
    event NFTMinted(
        uint256 indexed tokenId, 
        address indexed to, 
        string metadataUri,
        uint32 originChain
    );

    event NFTBurnedForTransfer(
        uint256 indexed tokenId,
        address indexed from,
        bytes32 indexed messageId,
        uint32 destinationChain,
        bytes32 recipient
    );

    event CrossChainNFTReceived(
        uint256 indexed tokenId,
        bytes32 indexed messageId,
        address indexed recipient,
        uint32 originChain,
        string metadataUri
    );

    event RevertEvent(string message, RevertContext revertContext);

    // Custom errors
    error Unauthorized();
    error InvalidChain();
    error MessageAlreadyProcessed();
    error InvalidMessage();
    error TransferFailed();
    error TokenNotFound();

    modifier onlyGateway() {
        if (msg.sender != address(gateway)) revert Unauthorized();
        _;
    }

    constructor(
        address payable gatewayAddress,
        address initialOwner,
        address _zetaChainUniversalNFT
    ) 
        ERC721("Universal NFT Receiver", "UNFTR") 
        Ownable(initialOwner)
    {
        gateway = GatewayEVM(gatewayAddress);
        zetaChainUniversalNFT = _zetaChainUniversalNFT;
    }

    // ============ Core NFT Functions ============

    /**
     * @notice Mint a new NFT on Base Sepolia
     * @param to Recipient address
     * @param metadataUri IPFS/HTTP URL for NFT metadata
     */
    function mint(address to, string calldata metadataUri) external whenNotPaused returns (uint256) {
        require(bytes(metadataUri).length > 0 && bytes(metadataUri).length <= MAX_METADATA_URI_LENGTH, "Invalid metadata URI");
        
        uint256 tokenId = _tokenIdCounter++;
        _safeMint(to, tokenId);
        _setTokenURI(tokenId, metadataUri);

        // Store origin information
        tokenOrigins[tokenId] = CrossChainMessage.NFTTransferMessage({
            tokenId: tokenId,
            metadataUri: metadataUri,
            recipient: bytes32(uint256(uint160(to))),
            originChain: BASE_SEPOLIA,
            destinationChain: 0, // Not applicable for minting
            messageId: bytes32(0),
            timestamp: uint64(block.timestamp),
            originContract: bytes32(uint256(uint160(address(this)))),
            nonce: 0
        });

        emit NFTMinted(tokenId, to, metadataUri, BASE_SEPOLIA);
        return tokenId;
    }

    /**
     * @notice Burn NFT and initiate cross-chain transfer to ZetaChain or Solana
     * @param tokenId Token to transfer
     * @param destinationChain Target chain ID (ZetaChain or Solana)
     * @param recipient Recipient address on destination chain (bytes32 format)
     */
    function _burnAndTransfer(
        uint256 tokenId,
        uint32 destinationChain,
        bytes32 recipient
    ) internal {
        require(_isValidDestinationChain(destinationChain), "Invalid destination chain");
        require(ownerOf(tokenId) == msg.sender, "Not token owner");
        
        string memory metadataUri = tokenURI(tokenId);
        CrossChainMessage.NFTTransferMessage memory originInfo = tokenOrigins[tokenId];
        
        // Generate unique message
        uint256 nonce = userNonces[msg.sender]++;
        bytes32 messageId = CrossChainMessage.generateMessageId(
            msg.sender,
            tokenId,
            destinationChain,
            nonce
        );

        // Create cross-chain message
        CrossChainMessage.NFTTransferMessage memory message = CrossChainMessage.NFTTransferMessage({
            tokenId: tokenId,
            metadataUri: metadataUri,
            recipient: recipient,
            originChain: originInfo.originChain, // Preserve original chain
            destinationChain: destinationChain,
            messageId: messageId,
            timestamp: uint64(block.timestamp),
            originContract: originInfo.originContract, // Preserve original contract
            nonce: nonce
        });

        // Validate and encode message
        CrossChainMessage.validateMessage(message, MAX_MESSAGE_AGE);
        bytes memory encodedMessage = CrossChainMessage.encode(message);

        // Burn the NFT
        _burn(tokenId);
        delete tokenOrigins[tokenId];

        // Send cross-chain message
        _sendCrossChainMessage(encodedMessage);

        emit NFTBurnedForTransfer(tokenId, msg.sender, messageId, destinationChain, recipient);
    }

    /**
     * @notice Public wrapper for burn and transfer functionality
     * @param tokenId Token to transfer
     * @param destinationChain Target chain ID (ZetaChain or Solana)  
     * @param recipient Recipient address on destination chain (bytes32 format)
     */
    function burnAndTransfer(
        uint256 tokenId,
        uint32 destinationChain,
        bytes32 recipient
    ) external payable nonReentrant whenNotPaused {
        _burnAndTransfer(tokenId, destinationChain, recipient);
    }

    /**
     * @notice Burn NFT and transfer to ZetaChain (convenience function with ETH address)
     * @param tokenId Token to transfer
     * @param recipient Recipient address on ZetaChain
     */
    function burnAndTransferToZetaChain(
        uint256 tokenId,
        address recipient
    ) external payable nonReentrant whenNotPaused {
        _burnAndTransfer(
            tokenId, 
            ZETACHAIN_TESTNET, 
            bytes32(uint256(uint160(recipient)))
        );
    }

    /**
     * @notice Burn NFT and transfer to Solana (convenience function with Solana pubkey)
     * @param tokenId Token to transfer
     * @param recipientPubkey Solana recipient public key (32 bytes)
     */
    function burnAndTransferToSolana(
        uint256 tokenId,
        bytes32 recipientPubkey
    ) external payable nonReentrant whenNotPaused {
        _burnAndTransfer(tokenId, SOLANA_DEVNET, recipientPubkey);
    }

    // ============ Cross-Chain Message Handling ============

    /**
     * @notice Handle incoming cross-chain call (from Gateway)
     * @param context Message context containing sender information
     * @param message Encoded NFT transfer message
     */
    function onCall(
        MessageContext calldata context,
        bytes calldata message
    ) external payable onlyGateway returns (bytes4) {
        CrossChainMessage.NFTTransferMessage memory nftMessage = CrossChainMessage.decode(message);
        
        // Validate message
        require(!processedMessages[nftMessage.messageId], "Message already processed");
        CrossChainMessage.validateMessage(nftMessage, MAX_MESSAGE_AGE);
        
        // Mark message as processed
        processedMessages[nftMessage.messageId] = true;

        // Convert recipient from bytes32 to address
        address recipient = address(uint160(uint256(nftMessage.recipient)));
        
        // Mint NFT to recipient
        uint256 tokenId = _tokenIdCounter++;
        _safeMint(recipient, tokenId);
        _setTokenURI(tokenId, nftMessage.metadataUri);

        // Store origin information (preserve provenance)
        tokenOrigins[tokenId] = nftMessage;

        emit CrossChainNFTReceived(
            tokenId,
            nftMessage.messageId,
            recipient,
            nftMessage.originChain,
            nftMessage.metadataUri
        );

        return this.onCall.selector;
    }

    /**
     * @notice Handle revert from cross-chain transaction
     * @param revertContext Revert context from Gateway
     */
    function onRevert(RevertContext calldata revertContext) external onlyGateway {
        emit RevertEvent("Cross-chain NFT transfer reverted", revertContext);
        
        // In production, implement NFT restoration logic here
        // This would require storing pre-burn state and restoring the NFT
    }

    // ============ Internal Functions ============

    /**
     * @notice Send cross-chain message via Gateway to ZetaChain
     */
    function _sendCrossChainMessage(bytes memory message) internal {
        // Set up revert options
        RevertOptions memory revertOptions = RevertOptions({
            revertAddress: address(this),
            callOnRevert: true,
            abortAddress: address(0),
            revertMessage: abi.encode("UniversalNFT transfer failed"),
            onRevertGasLimit: 100000
        });

        // Send message to ZetaChain Universal NFT contract  
        gateway.call(
            zetaChainUniversalNFT,
            message,
            revertOptions
        );
    }

    /**
     * @notice Check if destination chain is supported
     */
    function _isValidDestinationChain(uint32 chainId) internal pure returns (bool) {
        return chainId == ZETACHAIN_TESTNET || chainId == SOLANA_DEVNET;
    }

    // ============ Gateway Integration Functions ============

    /**
     * @notice Deposit ETH to ZetaChain with message
     * @param recipient Recipient on ZetaChain
     * @param message Cross-chain message
     */
    function depositAndCallZetaChain(
        address recipient,
        bytes calldata message
    ) external payable whenNotPaused {
        RevertOptions memory revertOptions = RevertOptions({
            revertAddress: address(this),
            callOnRevert: true,
            abortAddress: address(0),
            revertMessage: abi.encode("Deposit failed"),
            onRevertGasLimit: 100000
        });

        gateway.depositAndCall{value: msg.value}(
            recipient,
            message,
            revertOptions
        );
    }

    /**
     * @notice Deposit ERC20 tokens to ZetaChain with message
     * @param recipient Recipient on ZetaChain  
     * @param amount Amount of tokens
     * @param asset ERC20 token address
     * @param message Cross-chain message
     */
    function depositAndCallZetaChainERC20(
        address recipient,
        uint256 amount,
        address asset,
        bytes calldata message
    ) external whenNotPaused {
        IERC20(asset).safeTransferFrom(msg.sender, address(this), amount);
        IERC20(asset).approve(address(gateway), amount);

        RevertOptions memory revertOptions = RevertOptions({
            revertAddress: address(this),
            callOnRevert: true,
            abortAddress: address(0),
            revertMessage: abi.encode("ERC20 deposit failed"),
            onRevertGasLimit: 100000
        });

        gateway.depositAndCall(
            recipient,
            amount,
            asset,
            message,
            revertOptions
        );
    }

    // ============ View Functions ============

    /**
     * @notice Get token origin information
     */
    function getTokenOrigin(uint256 tokenId) external view returns (CrossChainMessage.NFTTransferMessage memory) {
        require(_ownerOf(tokenId) != address(0), "Token does not exist");
        return tokenOrigins[tokenId];
    }

    /**
     * @notice Check if message has been processed
     */
    function isMessageProcessed(bytes32 messageId) external view returns (bool) {
        return processedMessages[messageId];
    }

    /**
     * @notice Get user's current nonce
     */
    function getUserNonce(address user) external view returns (uint256) {
        return userNonces[user];
    }

    /**
     * @notice Get current token ID counter
     */
    function getCurrentTokenId() external view returns (uint256) {
        return _tokenIdCounter;
    }

    /**
     * @notice Get ZetaChain Universal NFT contract address
     */
    function getZetaChainContract() external view returns (address) {
        return zetaChainUniversalNFT;
    }

    // ============ Admin Functions ============

    /**
     * @notice Set ZetaChain Universal NFT contract address
     */
    function setZetaChainContract(address _zetaChainUniversalNFT) external onlyOwner {
        zetaChainUniversalNFT = _zetaChainUniversalNFT;
    }

    /**
     * @notice Pause contract operations
     */
    function pause() external onlyOwner {
        _pause();
    }

    /**
     * @notice Unpause contract operations
     */
    function unpause() external onlyOwner {
        _unpause();
    }

    /**
     * @notice Emergency function to mark message as processed (admin only)
     */
    function adminMarkMessageProcessed(bytes32 messageId) external onlyOwner {
        processedMessages[messageId] = true;
    }

    /**
     * @notice Withdraw accidentally sent ETH
     */
    function withdrawETH() external onlyOwner {
        payable(owner()).transfer(address(this).balance);
    }

    /**
     * @notice Withdraw accidentally sent ERC20 tokens
     */
    function withdrawERC20(address token) external onlyOwner {
        IERC20(token).safeTransfer(owner(), IERC20(token).balanceOf(address(this)));
    }

    // ============ Required Overrides ============

    function tokenURI(uint256 tokenId)
        public
        view
        override(ERC721, ERC721URIStorage)
        returns (string memory)
    {
        return super.tokenURI(tokenId);
    }

    function supportsInterface(bytes4 interfaceId)
        public
        view
        override(ERC721, ERC721URIStorage)
        returns (bool)
    {
        return super.supportsInterface(interfaceId);
    }

    /**
     * @notice Override _update to add pause functionality
     */
    function _update(address to, uint256 tokenId, address auth)
        internal
        override
        whenNotPaused
        returns (address)
    {
        return super._update(to, tokenId, auth);
    }

    /**
     * @notice Fallback to receive ETH
     */
    receive() external payable {}

    /**
     * @notice Fallback function
     */
    fallback() external payable {}
}