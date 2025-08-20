// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import {RevertContext, RevertOptions} from "@zetachain/protocol-contracts/contracts/Revert.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/UniversalContract.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/IGatewayZEVM.sol";
import "@zetachain/protocol-contracts/contracts/zevm/interfaces/IZRC20.sol";
import "@zetachain/protocol-contracts/contracts/zevm/GatewayZEVM.sol";
import "../../shared/CrossChainMessage.sol";

/**
 * @title UniversalNFT
 * @notice Universal NFT contract for ZetaChain that enables cross-chain NFT transfers
 * @dev Implements ERC721 with cross-chain functionality via ZetaChain Gateway
 */
contract UniversalNFT is 
    ERC721, 
    ERC721URIStorage, 
    Ownable, 
    ReentrancyGuard, 
    Pausable, 
    UniversalContract 
{
    using CrossChainMessage for CrossChainMessage.NFTTransferMessage;

    GatewayZEVM public immutable gateway;
    
    // Chain IDs
    uint32 public constant SOLANA_DEVNET = 900;
    uint32 public constant ZETACHAIN_TESTNET = 7001;
    uint32 public constant BASE_SEPOLIA = 84532;

    // Message validation
    uint256 public constant MAX_MESSAGE_AGE = 24 hours;
    uint256 public constant MAX_METADATA_URI_LENGTH = 500;

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
    event AbortEvent(string message, AbortContext abortContext);

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
        address initialOwner
    ) 
        ERC721("Universal NFT", "UNFT") 
        Ownable(initialOwner)
    {
        gateway = GatewayZEVM(gatewayAddress);
    }

    // ============ Core NFT Functions ============

    /**
     * @notice Mint a new NFT on ZetaChain
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
            originChain: ZETACHAIN_TESTNET,
            destinationChain: 0, // Not applicable for minting
            messageId: bytes32(0),
            timestamp: uint64(block.timestamp),
            originContract: bytes32(uint256(uint160(address(this)))),
            nonce: 0
        });

        emit NFTMinted(tokenId, to, metadataUri, ZETACHAIN_TESTNET);
        return tokenId;
    }

    /**
     * @notice Burn NFT and initiate cross-chain transfer
     * @param tokenId Token to transfer
     * @param destinationChain Target chain ID  
     * @param recipient Recipient address on destination chain
     * @param zrc20 ZRC20 token for paying gas fees
     */
    function burnAndTransfer(
        uint256 tokenId,
        uint32 destinationChain,
        bytes32 recipient,
        address zrc20
    ) external nonReentrant whenNotPaused {
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
        _sendCrossChainMessage(
            destinationChain,
            zrc20,
            encodedMessage,
            msg.sender
        );

        emit NFTBurnedForTransfer(tokenId, msg.sender, messageId, destinationChain, recipient);
    }

    // ============ Cross-Chain Message Handling ============

    /**
     * @notice Handle incoming cross-chain call (from Gateway)
     * @param context Message context
     * @param zrc20 ZRC20 token used for the transaction
     * @param amount Amount of tokens
     * @param message Encoded NFT transfer message
     */
    function onCall(
        MessageContext calldata context,
        address zrc20,
        uint256 amount,
        bytes calldata message
    ) external override onlyGateway {
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
    }

    /**
     * @notice Handle revert from cross-chain transaction
     * @param revertContext Revert context from Gateway
     */
    function onRevert(RevertContext calldata revertContext) external onlyGateway {
        // Log revert event for debugging
        emit RevertEvent("Cross-chain NFT transfer reverted", revertContext);
        
        // In a production system, you might want to:
        // 1. Restore the burned NFT to the original sender
        // 2. Refund gas fees
        // 3. Emit specific revert events for frontend handling
        
        // For this implementation, we'll just emit the event
        // The actual restoration logic would depend on storing pre-burn state
    }

    /**
     * @notice Handle abort from cross-chain transaction
     * @param abortContext Abort context from Gateway
     */
    function onAbort(AbortContext calldata abortContext) external onlyGateway {
        emit AbortEvent("Cross-chain NFT transfer aborted", abortContext);
    }

    // ============ Internal Functions ============

    /**
     * @notice Send cross-chain message via Gateway
     */
    function _sendCrossChainMessage(
        uint32 destinationChain,
        address zrc20,
        bytes memory message,
        address sender
    ) internal {
        // Get gas fee for the destination chain
        (, uint256 gasFee) = IZRC20(zrc20).withdrawGasFee();
        
        // Transfer gas fee from sender
        require(
            IZRC20(zrc20).transferFrom(sender, address(this), gasFee),
            "Gas fee transfer failed"
        );
        
        // Approve gateway to spend gas fee
        IZRC20(zrc20).approve(address(gateway), gasFee);

        // Get receiver contract address for destination chain
        bytes memory receiver = _getReceiverContract(destinationChain);

        // Set up call options
        CallOptions memory callOptions = CallOptions({
            gasLimit: 500000,
            isArbitraryCall: false
        });

        // Set up revert options
        RevertOptions memory revertOptions = RevertOptions({
            revertAddress: address(this),
            callOnRevert: true,
            abortAddress: address(0),
            revertMessage: abi.encode("UniversalNFT transfer failed"),
            onRevertGasLimit: 100000
        });

        // Send the message
        gateway.call(receiver, zrc20, message, callOptions, revertOptions);
    }

    /**
     * @notice Get receiver contract address for destination chain
     */
    function _getReceiverContract(uint32 chainId) internal pure returns (bytes memory) {
        if (chainId == BASE_SEPOLIA) {
            // Return Base Sepolia UniversalNFTReceiver contract address
            // This would be set during deployment
            return abi.encodePacked(address(0)); // Placeholder
        } else if (chainId == SOLANA_DEVNET) {
            // Return Solana program address as bytes
            // This would be the Solana Universal NFT program address
            return abi.encodePacked(bytes32(0)); // Placeholder
        }
        revert InvalidChain();
    }

    /**
     * @notice Check if destination chain is supported
     */
    function _isValidDestinationChain(uint32 chainId) internal pure returns (bool) {
        return chainId == BASE_SEPOLIA || chainId == SOLANA_DEVNET;
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

    // ============ Admin Functions ============

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
}