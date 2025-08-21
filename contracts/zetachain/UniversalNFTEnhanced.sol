// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "./UniversalNFT.sol";
import "../shared/ErrorRecoveryManager.sol";
import "../shared/ErrorHandling.sol";

/**
 * @title UniversalNFTEnhanced
 * @notice Enhanced Universal NFT contract with comprehensive error handling and recovery
 * @dev Extends UniversalNFT with timeout, retry, and admin recovery capabilities
 */
contract UniversalNFTEnhanced is UniversalNFT {
    using ErrorHandling for ErrorHandling.PendingMessage;

    // ============ Error Recovery Integration ============

    ErrorRecoveryManager public immutable errorRecoveryManager;
    
    // Enhanced state tracking
    mapping(bytes32 => uint256) public messageToTokenId; // messageId => tokenId (for recovery)
    mapping(uint256 => bool) public tokenInTransfer; // tokenId => inTransfer status
    mapping(bytes32 => uint256) public burnedTokenBackup; // messageId => tokenId (for revert)
    
    // Monitoring settings
    bool public monitoringEnabled = true;
    uint256 public autoRetryInterval = 1 hours;
    uint256 public lastAutoRetryRun;

    // ============ Enhanced Events ============

    event ErrorRecoveryManagerSet(address indexed manager, address indexed admin);
    event TransferMonitoringStarted(bytes32 indexed messageId, uint256 indexed tokenId, address indexed sender);
    event TransferCompleted(bytes32 indexed messageId, uint256 indexed newTokenId, address indexed recipient);
    event TransferReverted(bytes32 indexed messageId, uint256 indexed restoredTokenId, address indexed originalOwner);
    event AutoRetryTriggered(uint256 messagesProcessed, uint256 timestamp);
    event MonitoringStatusChanged(bool enabled, address indexed admin);

    // ============ Constructor ============

    constructor(
        address payable gatewayAddress,
        address initialOwner,
        address _errorRecoveryManager
    ) UniversalNFT(gatewayAddress, initialOwner) {
        require(_errorRecoveryManager != address(0), "Invalid recovery manager");
        errorRecoveryManager = ErrorRecoveryManager(_errorRecoveryManager);
        
        emit ErrorRecoveryManagerSet(_errorRecoveryManager, initialOwner);
    }

    // ============ Enhanced Transfer Functions ============

    /**
     * @notice Enhanced burn and transfer with comprehensive error handling
     * @param tokenId Token to transfer
     * @param destinationChain Target chain ID  
     * @param recipient Recipient address on destination chain
     * @param zrc20 ZRC20 token for paying gas fees
     */
    function burnAndTransferEnhanced(
        uint256 tokenId,
        uint32 destinationChain,
        bytes32 recipient,
        address zrc20
    ) external nonReentrant whenNotPaused {
        require(_isValidDestinationChain(destinationChain), "Invalid destination chain");
        require(ownerOf(tokenId) == msg.sender, "Not token owner");
        require(!tokenInTransfer[tokenId], "Token already in transfer");
        
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
            originChain: originInfo.originChain,
            destinationChain: destinationChain,
            messageId: messageId,
            timestamp: uint64(block.timestamp),
            originContract: originInfo.originContract,
            nonce: nonce
        });

        // Register with error recovery system
        if (monitoringEnabled) {
            try errorRecoveryManager.registerMessage(message, msg.sender) {
                emit TransferMonitoringStarted(messageId, tokenId, msg.sender);
            } catch {
                // Continue even if monitoring registration fails
                ErrorHandling.logSimpleError(
                    messageId,
                    "MONITORING_ERROR",
                    "Failed to register message for monitoring"
                );
            }
        }

        // Mark token as in transfer and backup for potential revert
        tokenInTransfer[tokenId] = true;
        messageToTokenId[messageId] = tokenId;
        burnedTokenBackup[messageId] = tokenId;

        // Validate and encode message
        CrossChainMessage.validateMessage(message, MAX_MESSAGE_AGE);
        bytes memory encodedMessage = CrossChainMessage.encode(message);

        // Burn the NFT
        _burn(tokenId);
        delete tokenOrigins[tokenId];

        // Attempt to send cross-chain message
        try this._sendCrossChainMessageEnhanced(
            destinationChain,
            zrc20,
            encodedMessage,
            msg.sender,
            messageId
        ) {
            emit NFTBurnedForTransfer(tokenId, msg.sender, messageId, destinationChain, recipient);
        } catch Error(string memory reason) {
            _handleTransferFailure(messageId, tokenId, msg.sender, metadataUri, reason);
            revert(reason);
        } catch {
            _handleTransferFailure(messageId, tokenId, msg.sender, metadataUri, "Unknown transfer error");
            revert("Transfer failed");
        }
    }

    /**
     * @notice Enhanced cross-chain message sending with error handling
     */
    function _sendCrossChainMessageEnhanced(
        uint32 destinationChain,
        address zrc20,
        bytes memory message,
        address sender,
        bytes32 messageId
    ) external {
        require(msg.sender == address(this), "Internal function only");
        
        try this._executeCrossChainSend(destinationChain, zrc20, message, sender) {
            // Success - message sent
            if (monitoringEnabled) {
                // Start timeout monitoring
                // The recovery manager will handle timeouts automatically
            }
        } catch Error(string memory reason) {
            // Report error to recovery system
            if (monitoringEnabled) {
                errorRecoveryManager.reportMessageError(
                    messageId,
                    "SEND_ERROR",
                    reason,
                    true // shouldRetry
                );
            }
            revert(reason);
        } catch {
            // Report generic error
            if (monitoringEnabled) {
                errorRecoveryManager.reportMessageError(
                    messageId,
                    "SEND_ERROR",
                    "Unknown send error",
                    true // shouldRetry
                );
            }
            revert("Send failed");
        }
    }

    /**
     * @notice Internal function to execute cross-chain send
     */
    function _executeCrossChainSend(
        uint32 destinationChain,
        address zrc20,
        bytes memory message,
        address sender
    ) external {
        require(msg.sender == address(this), "Internal function only");
        _sendCrossChainMessage(destinationChain, zrc20, message, sender);
    }

    /**
     * @notice Handle transfer failure and attempt recovery
     */
    function _handleTransferFailure(
        bytes32 messageId,
        uint256 tokenId,
        address originalOwner,
        string memory metadataUri,
        string memory reason
    ) internal {
        // Restore the token to original owner
        _safeMint(originalOwner, tokenId);
        _setTokenURI(tokenId, metadataUri);
        
        // Clear transfer state
        tokenInTransfer[tokenId] = false;
        delete messageToTokenId[messageId];
        delete burnedTokenBackup[messageId];
        
        // Log error
        ErrorHandling.logError(
            messageId,
            "TRANSFER_FAILURE",
            reason,
            abi.encode(tokenId, originalOwner, metadataUri)
        );

        emit TransferReverted(messageId, tokenId, originalOwner);
    }

    // ============ Enhanced Message Handling ============

    /**
     * @notice Enhanced cross-chain message handler with monitoring
     */
    function onCall(
        MessageContext calldata context,
        address zrc20,
        uint256 amount,
        bytes calldata message
    ) external override onlyGateway {
        CrossChainMessage.NFTTransferMessage memory nftMessage = CrossChainMessage.decode(message);
        
        // Check if message already processed
        require(!processedMessages[nftMessage.messageId], "Message already processed");
        
        try this._processIncomingMessage(nftMessage, context) {
            // Success - mark as completed in recovery system
            if (monitoringEnabled) {
                try errorRecoveryManager.markMessageCompleted(nftMessage.messageId) {
                    // Success
                } catch {
                    // Continue even if monitoring update fails
                }
            }
        } catch Error(string memory reason) {
            // Report processing error
            if (monitoringEnabled) {
                errorRecoveryManager.reportMessageError(
                    nftMessage.messageId,
                    "PROCESSING_ERROR",
                    reason,
                    false // Don't auto-retry processing errors
                );
            }
            revert(reason);
        }
    }

    /**
     * @notice Internal message processing with detailed error handling
     */
    function _processIncomingMessage(
        CrossChainMessage.NFTTransferMessage memory nftMessage,
        MessageContext calldata context
    ) external {
        require(msg.sender == address(this), "Internal function only");
        
        // Validate message
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

        emit TransferCompleted(nftMessage.messageId, tokenId, recipient);
    }

    // ============ Enhanced Revert Handling ============

    /**
     * @notice Enhanced revert handling with automatic recovery
     */
    function onRevert(RevertContext calldata revertContext) external override onlyGateway {
        emit RevertEvent("Cross-chain NFT transfer reverted", revertContext);
        
        // Attempt to extract message ID from revert context
        bytes32 messageId = _extractMessageIdFromRevert(revertContext);
        
        if (messageId != bytes32(0)) {
            _handleTransferRevert(messageId, revertContext);
        } else {
            ErrorHandling.logError(
                messageId,
                "REVERT_ERROR",
                "Could not extract message ID from revert context",
                abi.encode(revertContext)
            );
        }
    }

    /**
     * @notice Handle transfer revert with automatic token restoration
     */
    function _handleTransferRevert(
        bytes32 messageId,
        RevertContext calldata revertContext
    ) internal {
        uint256 tokenId = burnedTokenBackup[messageId];
        
        if (tokenId > 0) {
            // Get original token info
            address originalOwner = tokenOrigins[tokenId].recipient != bytes32(0) 
                ? address(uint160(uint256(tokenOrigins[tokenId].recipient)))
                : msg.sender; // Fallback to sender
            
            string memory metadataUri = tokenOrigins[tokenId].metadataUri;
            
            // Restore the token
            _safeMint(originalOwner, tokenId);
            _setTokenURI(tokenId, metadataUri);
            
            // Clear transfer state
            tokenInTransfer[tokenId] = false;
            delete messageToTokenId[messageId];
            delete burnedTokenBackup[messageId];
            
            emit TransferReverted(messageId, tokenId, originalOwner);
            
            // Update recovery system
            if (monitoringEnabled) {
                errorRecoveryManager.reportMessageError(
                    messageId,
                    "REVERT",
                    "Transfer reverted by gateway",
                    false // Don't retry reverted transfers
                );
            }
        }
    }

    /**
     * @notice Extract message ID from revert context (implementation specific)
     */
    function _extractMessageIdFromRevert(
        RevertContext calldata revertContext
    ) internal view returns (bytes32 messageId) {
        // This would depend on how the revert context is structured
        // For now, we'll try to decode the revert message
        try this._parseRevertMessage(revertContext.revertMessage) returns (bytes32 parsedId) {
            return parsedId;
        } catch {
            return bytes32(0);
        }
    }

    /**
     * @notice Parse revert message to extract message ID
     */
    function _parseRevertMessage(bytes memory revertMessage) external pure returns (bytes32 messageId) {
        if (revertMessage.length >= 32) {
            assembly {
                messageId := mload(add(revertMessage, 32))
            }
        }
        return messageId;
    }

    // ============ Automated Monitoring Functions ============

    /**
     * @notice Run automated retry processing
     * @return processedCount Number of messages processed
     */
    function runAutoRetry() external returns (uint256 processedCount) {
        require(
            block.timestamp >= lastAutoRetryRun + autoRetryInterval,
            "Auto retry interval not reached"
        );
        
        lastAutoRetryRun = block.timestamp;
        
        if (monitoringEnabled) {
            try errorRecoveryManager.executeAutoRetries(20) returns (uint256 processed) {
                processedCount = processed;
                emit AutoRetryTriggered(processed, block.timestamp);
            } catch {
                // Continue even if auto-retry fails
                ErrorHandling.logSimpleError(
                    bytes32(0),
                    "AUTO_RETRY_ERROR",
                    "Auto retry execution failed"
                );
            }
        }
        
        return processedCount;
    }

    /**
     * @notice Check for timed out messages and handle them
     * @param maxMessages Maximum number of messages to check
     * @return handledCount Number of timeouts handled
     */
    function handleTimeouts(uint256 maxMessages) external returns (uint256 handledCount) {
        if (!monitoringEnabled) {
            return 0;
        }
        
        // Get recent message IDs to check (implementation would vary)
        bytes32[] memory recentMessages = _getRecentMessages(maxMessages);
        
        try errorRecoveryManager.handleTimeouts(recentMessages) returns (uint256 handled) {
            return handled;
        } catch {
            ErrorHandling.logSimpleError(
                bytes32(0),
                "TIMEOUT_HANDLING_ERROR",
                "Timeout handling failed"
            );
            return 0;
        }
    }

    /**
     * @notice Get recent messages for timeout checking
     */
    function _getRecentMessages(uint256 maxMessages) internal view returns (bytes32[] memory messageIds) {
        // This would be implemented based on how messages are tracked
        // For now, return empty array
        return new bytes32[](0);
    }

    // ============ Admin Functions ============

    /**
     * @notice Set monitoring status
     * @param enabled Whether monitoring should be enabled
     */
    function setMonitoringEnabled(bool enabled) external onlyOwner {
        monitoringEnabled = enabled;
        emit MonitoringStatusChanged(enabled, msg.sender);
    }

    /**
     * @notice Set auto retry interval
     * @param interval New interval in seconds
     */
    function setAutoRetryInterval(uint256 interval) external onlyOwner {
        require(interval >= 5 minutes, "Interval too short");
        require(interval <= 24 hours, "Interval too long");
        autoRetryInterval = interval;
    }

    /**
     * @notice Admin function to recover stuck token
     * @param messageId Message ID of stuck transfer
     * @param tokenId Token ID to recover
     * @param originalOwner Original owner to restore to
     * @param metadataUri Original metadata URI
     */
    function adminRecoverToken(
        bytes32 messageId,
        uint256 tokenId,
        address originalOwner,
        string calldata metadataUri
    ) external onlyOwner {
        require(tokenInTransfer[tokenId], "Token not in transfer");
        require(burnedTokenBackup[messageId] == tokenId, "Message-token mismatch");
        
        // Restore the token
        _safeMint(originalOwner, tokenId);
        _setTokenURI(tokenId, metadataUri);
        
        // Clear transfer state
        tokenInTransfer[tokenId] = false;
        delete messageToTokenId[messageId];
        delete burnedTokenBackup[messageId];
        
        emit TransferReverted(messageId, tokenId, originalOwner);
    }

    // ============ View Functions ============

    /**
     * @notice Check if token is currently in transfer
     * @param tokenId Token ID to check
     * @return inTransfer Whether token is in transfer
     */
    function isTokenInTransfer(uint256 tokenId) external view returns (bool inTransfer) {
        return tokenInTransfer[tokenId];
    }

    /**
     * @notice Get message ID for a token in transfer
     * @param tokenId Token ID
     * @return messageId Associated message ID
     */
    function getTokenTransferMessage(uint256 tokenId) external view returns (bytes32 messageId) {
        // Linear search through messageToTokenId mapping
        // In production, this would be optimized with reverse mapping
        return bytes32(0); // Placeholder
    }

    /**
     * @notice Get monitoring status
     * @return enabled Whether monitoring is enabled
     * @return lastRetry Timestamp of last auto retry run
     * @return retryInterval Current auto retry interval
     */
    function getMonitoringStatus() external view returns (
        bool enabled,
        uint256 lastRetry,
        uint256 retryInterval
    ) {
        return (monitoringEnabled, lastAutoRetryRun, autoRetryInterval);
    }

    /**
     * @notice Get error recovery manager address
     * @return manager Address of the error recovery manager
     */
    function getErrorRecoveryManager() external view returns (address manager) {
        return address(errorRecoveryManager);
    }
}