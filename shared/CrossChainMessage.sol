// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

/**
 * @title CrossChainMessage
 * @notice Shared message format for Universal NFT Protocol cross-chain transfers
 * @dev This struct must be serializable/deserializable across Solana (Rust) and EVM (Solidity)
 */
library CrossChainMessage {
    struct NFTTransferMessage {
        uint256 tokenId;         // NFT token ID
        string metadataUri;      // IPFS/HTTP URL to NFT metadata
        bytes32 recipient;       // Recipient address (32 bytes to support both EVM and Solana)
        uint32 originChain;      // Chain ID where NFT was originally minted
        uint32 destinationChain; // Target chain ID
        bytes32 messageId;       // Unique message identifier for replay protection
        uint64 timestamp;        // Block timestamp when message was created
        bytes32 originContract;  // Original contract address where NFT was first minted
        uint256 nonce;          // Sender's nonce for additional uniqueness
    }

    /**
     * @notice Encode NFT transfer message for cross-chain transmission
     * @param message The NFT transfer message to encode
     * @return Encoded bytes suitable for Gateway transmission
     */
    function encode(NFTTransferMessage memory message) internal pure returns (bytes memory) {
        return abi.encode(
            message.tokenId,
            message.metadataUri,
            message.recipient,
            message.originChain,
            message.destinationChain,
            message.messageId,
            message.timestamp,
            message.originContract,
            message.nonce
        );
    }

    /**
     * @notice Decode NFT transfer message from cross-chain transmission
     * @param data The encoded message data
     * @return Decoded NFT transfer message
     */
    function decode(bytes calldata data) internal pure returns (NFTTransferMessage memory) {
        (
            uint256 tokenId,
            string memory metadataUri,
            bytes32 recipient,
            uint32 originChain,
            uint32 destinationChain,
            bytes32 messageId,
            uint64 timestamp,
            bytes32 originContract,
            uint256 nonce
        ) = abi.decode(data, (uint256, string, bytes32, uint32, uint32, bytes32, uint64, bytes32, uint256));

        return NFTTransferMessage({
            tokenId: tokenId,
            metadataUri: metadataUri,
            recipient: recipient,
            originChain: originChain,
            destinationChain: destinationChain,
            messageId: messageId,
            timestamp: timestamp,
            originContract: originContract,
            nonce: nonce
        });
    }

    /**
     * @notice Generate unique message ID
     * @param sender Address of the sender
     * @param tokenId Token ID being transferred
     * @param destinationChain Target chain ID
     * @param nonce Sender's nonce
     * @return Generated message ID
     */
    function generateMessageId(
        address sender,
        uint256 tokenId,
        uint32 destinationChain,
        uint256 nonce
    ) internal view returns (bytes32) {
        return keccak256(abi.encodePacked(
            sender,
            tokenId,
            destinationChain,
            nonce,
            block.timestamp
        ));
    }

    /**
     * @notice Validate message timestamp and size constraints
     * @param message The message to validate
     * @param maxAge Maximum age in seconds for messages
     */
    function validateMessage(NFTTransferMessage memory message, uint256 maxAge) internal view {
        require(message.timestamp > 0, "Invalid timestamp");
        require(block.timestamp - message.timestamp <= maxAge, "Message too old");
        require(bytes(message.metadataUri).length > 0, "Empty metadata URI");
        require(bytes(message.metadataUri).length <= 500, "Metadata URI too long");
        require(message.originChain != message.destinationChain, "Same chain transfer");
    }
}