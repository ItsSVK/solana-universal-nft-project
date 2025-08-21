// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/access/AccessControl.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "@openzeppelin/contracts/utils/Pausable.sol";
import "./ErrorHandling.sol";
import "../../shared/CrossChainMessage.sol";

/**
 * @title ErrorRecoveryManager
 * @notice Centralized error handling and recovery system for cross-chain NFT operations
 * @dev Manages timeouts, retries, admin recovery functions, and comprehensive logging
 */
contract ErrorRecoveryManager is Ownable, AccessControl, ReentrancyGuard, Pausable {
    using ErrorHandling for ErrorHandling.PendingMessage;
    using ErrorHandling for ErrorHandling.RetryConfig;
    using ErrorHandling for ErrorHandling.TimeoutConfig;

    // ============ Access Control Roles ============
    
    bytes32 public constant ADMIN_ROLE = keccak256("ADMIN_ROLE");
    bytes32 public constant OPERATOR_ROLE = keccak256("OPERATOR_ROLE");
    bytes32 public constant RECOVERY_ROLE = keccak256("RECOVERY_ROLE");

    // ============ State Variables ============

    // Message tracking
    mapping(bytes32 => ErrorHandling.PendingMessage) public pendingMessages;
    mapping(address => bytes32[]) public userMessages; // user => messageIds[]
    bytes32[] public allMessageIds;
    
    // Configuration
    ErrorHandling.RetryConfig public retryConfig;
    ErrorHandling.TimeoutConfig internal timeoutConfig;
    
    // Statistics
    uint256 public totalMessages;
    uint256 public completedMessages;
    uint256 public failedMessages;
    uint256 public timeoutMessages;
    uint256 public recoveredMessages;
    
    // Recovery settings
    bool public autoRetryEnabled = true;
    bool public adminRecoveryEnabled = true;
    uint256 public recoveryWindow = 7 days; // Time window for admin recovery
    
    // Contract addresses for recovery operations
    mapping(uint32 => address) public chainContracts; // chainId => contract address

    // ============ Events ============

    event RecoveryManagerInitialized(
        address indexed admin,
        uint256 defaultTimeout,
        uint256 maxRetries
    );

    event MessageRegistered(
        bytes32 indexed messageId,
        address indexed sender,
        uint32 indexed destinationChain,
        uint256 timeout
    );

    event AutoRetryExecuted(
        bytes32 indexed messageId,
        uint256 indexed attemptNumber,
        bool success,
        string result
    );

    event RecoveryConfigUpdated(
        string indexed configType,
        uint256 oldValue,
        uint256 newValue,
        address indexed updatedBy
    );

    event BatchRecoveryExecuted(
        bytes32[] messageIds,
        ErrorHandling.RecoveryAction indexed action,
        address indexed admin,
        uint256 successCount
    );

    event EmergencyRecoveryActivated(
        address indexed admin,
        string reason,
        uint256 affectedMessages
    );

    // ============ Modifiers ============

    modifier onlyValidMessage(bytes32 messageId) {
        require(pendingMessages[messageId].messageId != bytes32(0), "Message not found");
        _;
    }

    modifier onlyRecoveryWindow(bytes32 messageId) {
        require(
            block.timestamp <= pendingMessages[messageId].createdAt + recoveryWindow,
            "Recovery window expired"
        );
        _;
    }

    // ============ Constructor ============

    constructor(address initialOwner) Ownable(initialOwner) {
        _grantRole(DEFAULT_ADMIN_ROLE, initialOwner);
        _grantRole(ADMIN_ROLE, initialOwner);
        
        // Initialize default configurations
        retryConfig = ErrorHandling.RetryConfig({
            maxRetries: ErrorHandling.MAX_RETRY_ATTEMPTS,
            baseDelay: ErrorHandling.MIN_RETRY_DELAY,
            maxDelay: ErrorHandling.MAX_RETRY_DELAY,
            backoffMultiplier: ErrorHandling.RETRY_BACKOFF_MULTIPLIER,
            exponentialBackoff: true
        });

        timeoutConfig.defaultTimeout = ErrorHandling.DEFAULT_MESSAGE_TIMEOUT;
        timeoutConfig.extendedTimeout = ErrorHandling.EXTENDED_MESSAGE_TIMEOUT;
        timeoutConfig.useChainSpecificTimeouts = true;

        // Set chain-specific timeouts
        timeoutConfig.chainSpecificTimeouts[900] = 2 hours; // Solana
        timeoutConfig.chainSpecificTimeouts[7001] = 1 hours; // ZetaChain
        timeoutConfig.chainSpecificTimeouts[84532] = 1 hours; // Base Sepolia

        emit RecoveryManagerInitialized(
            initialOwner,
            ErrorHandling.DEFAULT_MESSAGE_TIMEOUT,
            ErrorHandling.MAX_RETRY_ATTEMPTS
        );
    }

    // ============ Core Message Management ============

    /**
     * @notice Register a new cross-chain message for monitoring
     * @param message Cross-chain NFT transfer message
     * @param sender Original sender of the message
     * @return success Whether registration was successful
     */
    function registerMessage(
        CrossChainMessage.NFTTransferMessage calldata message,
        address sender
    ) external whenNotPaused returns (bool success) {
        require(message.messageId != bytes32(0), "Invalid message ID");
        require(pendingMessages[message.messageId].messageId == bytes32(0), "Message already registered");

        uint256 timeout = ErrorHandling.calculateMessageTimeout(
            message.destinationChain,
            timeoutConfig
        );

        ErrorHandling.PendingMessage memory pendingMsg = ErrorHandling.createPendingMessage(
            message,
            sender,
            timeout
        );

        pendingMessages[message.messageId] = pendingMsg;
        userMessages[sender].push(message.messageId);
        allMessageIds.push(message.messageId);
        totalMessages++;

        emit MessageRegistered(
            message.messageId,
            sender,
            message.destinationChain,
            timeout
        );

        ErrorHandling.logSimpleError(
            message.messageId,
            "INFO",
            "Message registered for monitoring"
        );

        return true;
    }

    /**
     * @notice Mark message as completed
     * @param messageId Message ID to mark as completed
     * @return success Whether marking was successful
     */
    function markMessageCompleted(
        bytes32 messageId
    ) external onlyValidMessage(messageId) returns (bool success) {
        ErrorHandling.PendingMessage storage pendingMsg = pendingMessages[messageId];
        
        require(
            pendingMsg.status != ErrorHandling.MessageStatus.COMPLETED,
            "Message already completed"
        );

        ErrorHandling.MessageStatus oldStatus = pendingMsg.status;
        pendingMsg.status = ErrorHandling.MessageStatus.COMPLETED;
        completedMessages++;

        emit ErrorHandling.MessageStatusChanged(
            messageId,
            oldStatus,
            ErrorHandling.MessageStatus.COMPLETED,
            "Message processing completed successfully"
        );

        return true;
    }

    /**
     * @notice Report message processing error
     * @param messageId Message ID that failed
     * @param errorType Category of error
     * @param errorMessage Detailed error message
     * @param shouldRetry Whether automatic retry should be attempted
     */
    function reportMessageError(
        bytes32 messageId,
        string calldata errorType,
        string calldata errorMessage,
        bool shouldRetry
    ) external onlyValidMessage(messageId) {
        ErrorHandling.PendingMessage storage pendingMsg = pendingMessages[messageId];
        
        ErrorHandling.logSimpleError(messageId, errorType, errorMessage);
        
        if (shouldRetry && autoRetryEnabled && pendingMsg.isRecoverable) {
            ErrorHandling.handleRetryFailure(pendingMsg, errorMessage, retryConfig);
        } else {
            ErrorHandling.MessageStatus oldStatus = pendingMsg.status;
            pendingMsg.status = ErrorHandling.MessageStatus.FAILED;
            pendingMsg.lastError = errorMessage;
            failedMessages++;

            emit ErrorHandling.MessageStatusChanged(
                messageId,
                oldStatus,
                ErrorHandling.MessageStatus.FAILED,
                string(abi.encodePacked("Processing failed: ", errorMessage))
            );
        }
    }

    // ============ Timeout Management ============

    /**
     * @notice Check and handle message timeouts
     * @param messageIds Array of message IDs to check
     * @return handledCount Number of timeouts handled
     */
    function handleTimeouts(
        bytes32[] calldata messageIds
    ) external whenNotPaused returns (uint256 handledCount) {
        for (uint256 i = 0; i < messageIds.length; i++) {
            bytes32 messageId = messageIds[i];
            
            if (pendingMessages[messageId].messageId == bytes32(0)) {
                continue; // Skip non-existent messages
            }

            ErrorHandling.PendingMessage storage pendingMsg = pendingMessages[messageId];
            
            (bool shouldRetry, uint256 retryDelay) = ErrorHandling.handleMessageTimeout(
                pendingMsg,
                retryConfig
            );

            if (shouldRetry) {
                ErrorHandling.scheduleRetry(pendingMsg, retryConfig, "Timeout recovery");
            }

            if (pendingMsg.status == ErrorHandling.MessageStatus.TIMEOUT ||
                pendingMsg.status == ErrorHandling.MessageStatus.FAILED) {
                handledCount++;
                if (pendingMsg.status == ErrorHandling.MessageStatus.TIMEOUT) {
                    timeoutMessages++;
                }
            }
        }

        return handledCount;
    }

    /**
     * @notice Get timed out messages for a specific time range
     * @param fromTime Start time for search
     * @param toTime End time for search
     * @return timedOutMessages Array of timed out message IDs
     */
    function getTimedOutMessages(
        uint256 fromTime,
        uint256 toTime
    ) external view returns (bytes32[] memory timedOutMessages) {
        bytes32[] memory tempResults = new bytes32[](allMessageIds.length);
        uint256 count = 0;

        for (uint256 i = 0; i < allMessageIds.length; i++) {
            bytes32 messageId = allMessageIds[i];
            ErrorHandling.PendingMessage storage pendingMsg = pendingMessages[messageId];
            
            if (pendingMsg.createdAt >= fromTime && 
                pendingMsg.createdAt <= toTime) {
                
                (bool hasTimedOut, ) = ErrorHandling.isMessageTimedOut(pendingMsg);
                if (hasTimedOut) {
                    tempResults[count] = messageId;
                    count++;
                }
            }
        }

        // Resize array to actual count
        timedOutMessages = new bytes32[](count);
        for (uint256 j = 0; j < count; j++) {
            timedOutMessages[j] = tempResults[j];
        }

        return timedOutMessages;
    }

    // ============ Retry Management ============

    /**
     * @notice Execute automatic retries for ready messages
     * @param maxRetries Maximum number of retries to process in this call
     * @return processedCount Number of retries processed
     */
    function executeAutoRetries(
        uint256 maxRetries
    ) external whenNotPaused returns (uint256 processedCount) {
        uint256 processed = 0;
        
        for (uint256 i = 0; i < allMessageIds.length && processed < maxRetries; i++) {
            bytes32 messageId = allMessageIds[i];
            ErrorHandling.PendingMessage storage pendingMsg = pendingMessages[messageId];
            
            if (ErrorHandling.isReadyForRetry(pendingMsg)) {
                bool success = _executeRetry(messageId);
                
                emit AutoRetryExecuted(
                    messageId,
                    pendingMsg.retryCount,
                    success,
                    success ? "Retry successful" : pendingMsg.lastError
                );
                
                processed++;
            }
        }

        return processed;
    }

    /**
     * @notice Force retry a specific message (admin only)
     * @param messageId Message to retry
     * @param reason Reason for forced retry
     */
    function forceRetry(
        bytes32 messageId,
        string calldata reason
    ) external onlyRole(ADMIN_ROLE) onlyValidMessage(messageId) {
        ErrorHandling.PendingMessage storage pendingMsg = pendingMessages[messageId];
        require(pendingMsg.isRecoverable, "Message not recoverable");
        
        ErrorHandling.scheduleRetry(pendingMsg, retryConfig, reason);
    }

    /**
     * @notice Internal retry execution logic
     * @param messageId Message to retry
     * @return success Whether retry was successful
     */
    function _executeRetry(bytes32 messageId) internal returns (bool success) {
        // This would integrate with the actual cross-chain messaging system
        // For now, we'll simulate the retry logic
        
        ErrorHandling.PendingMessage storage pendingMsg = pendingMessages[messageId];
        pendingMsg.status = ErrorHandling.MessageStatus.PROCESSING;
        
        // In a real implementation, this would:
        // 1. Re-encode the message
        // 2. Send via the appropriate gateway
        // 3. Handle the response
        
        // For simulation, we'll assume 70% success rate
        success = (uint256(keccak256(abi.encodePacked(messageId, block.timestamp))) % 100) < 70;
        
        if (success) {
            pendingMsg.status = ErrorHandling.MessageStatus.COMPLETED;
            completedMessages++;
        } else {
            ErrorHandling.handleRetryFailure(
                pendingMsg,
                "Simulated retry failure",
                retryConfig
            );
        }
        
        return success;
    }

    // ============ Admin Recovery Functions ============

    /**
     * @notice Execute recovery action for a single message
     * @param messageId Message to recover
     * @param action Recovery action to perform
     * @param details Additional details about the recovery
     * @param recoveryHash Recovery verification hash (for high-risk actions)
     */
    function executeRecovery(
        bytes32 messageId,
        ErrorHandling.RecoveryAction action,
        string calldata details,
        bytes32 recoveryHash
    ) external onlyRole(RECOVERY_ROLE) onlyValidMessage(messageId) onlyRecoveryWindow(messageId) {
        require(adminRecoveryEnabled, "Admin recovery disabled");
        
        ErrorHandling.PendingMessage storage pendingMsg = pendingMessages[messageId];
        
        require(
            ErrorHandling.verifyRecovery(pendingMsg, action, msg.sender, recoveryHash),
            "Recovery verification failed"
        );

        ErrorHandling.executeRecovery(pendingMsg, action, msg.sender, details);
        recoveredMessages++;
    }

    /**
     * @notice Execute batch recovery for multiple messages
     * @param messageIds Array of message IDs to recover
     * @param action Recovery action to perform on all messages
     * @param details Recovery details
     */
    function executeBatchRecovery(
        bytes32[] calldata messageIds,
        ErrorHandling.RecoveryAction action,
        string calldata details
    ) external onlyRole(RECOVERY_ROLE) {
        require(adminRecoveryEnabled, "Admin recovery disabled");
        require(messageIds.length > 0, "No messages provided");
        require(messageIds.length <= 50, "Too many messages in batch");

        uint256 successCount = 0;

        for (uint256 i = 0; i < messageIds.length; i++) {
            bytes32 messageId = messageIds[i];
            
            if (pendingMessages[messageId].messageId == bytes32(0)) {
                continue; // Skip non-existent messages
            }

            if (block.timestamp > pendingMessages[messageId].createdAt + recoveryWindow) {
                continue; // Skip expired recovery window
            }

            try this.executeRecovery(messageId, action, details, bytes32(0)) {
                successCount++;
            } catch {
                // Continue with next message on failure
                ErrorHandling.logSimpleError(
                    messageId,
                    "RECOVERY_ERROR",
                    "Batch recovery failed for message"
                );
            }
        }

        emit BatchRecoveryExecuted(messageIds, action, msg.sender, successCount);
    }

    /**
     * @notice Emergency recovery function for critical situations
     * @param messageIds Messages to recover
     * @param reason Reason for emergency recovery
     */
    function emergencyRecovery(
        bytes32[] calldata messageIds,
        string calldata reason
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        require(messageIds.length > 0, "No messages provided");
        
        uint256 affectedCount = 0;

        for (uint256 i = 0; i < messageIds.length; i++) {
            bytes32 messageId = messageIds[i];
            
            if (pendingMessages[messageId].messageId == bytes32(0)) {
                continue;
            }

            ErrorHandling.PendingMessage storage pendingMsg = pendingMessages[messageId];
            
            if (!ErrorHandling.isStatusFinal(pendingMsg.status)) {
                pendingMsg.status = ErrorHandling.MessageStatus.ADMIN_RESOLVED;
                pendingMsg.isRecoverable = false;
                affectedCount++;
            }
        }

        recoveredMessages += affectedCount;

        emit EmergencyRecoveryActivated(msg.sender, reason, affectedCount);
    }

    // ============ Configuration Management ============

    /**
     * @notice Update retry configuration
     * @param maxRetries Maximum retry attempts
     * @param baseDelay Base delay between retries
     * @param maxDelay Maximum delay between retries
     * @param backoffMultiplier Exponential backoff multiplier
     * @param useExponentialBackoff Whether to use exponential backoff
     */
    function updateRetryConfig(
        uint256 maxRetries,
        uint256 baseDelay,
        uint256 maxDelay,
        uint256 backoffMultiplier,
        bool useExponentialBackoff
    ) external onlyRole(ADMIN_ROLE) {
        require(maxRetries <= 20, "Max retries too high");
        require(baseDelay >= 1 minutes, "Base delay too low");
        require(maxDelay <= 24 hours, "Max delay too high");
        require(backoffMultiplier >= 1, "Invalid backoff multiplier");

        retryConfig.maxRetries = maxRetries;
        retryConfig.baseDelay = baseDelay;
        retryConfig.maxDelay = maxDelay;
        retryConfig.backoffMultiplier = backoffMultiplier;
        retryConfig.exponentialBackoff = useExponentialBackoff;

        emit RecoveryConfigUpdated("RETRY_MAX_ATTEMPTS", retryConfig.maxRetries, maxRetries, msg.sender);
    }

    /**
     * @notice Update timeout configuration
     * @param defaultTimeout Default message timeout
     * @param extendedTimeout Extended timeout for complex chains
     */
    function updateTimeoutConfig(
        uint256 defaultTimeout,
        uint256 extendedTimeout
    ) external onlyRole(ADMIN_ROLE) {
        require(defaultTimeout >= 10 minutes, "Default timeout too low");
        require(extendedTimeout >= defaultTimeout, "Extended timeout must be >= default");
        require(extendedTimeout <= 7 days, "Extended timeout too high");

        timeoutConfig.defaultTimeout = defaultTimeout;
        timeoutConfig.extendedTimeout = extendedTimeout;

        emit RecoveryConfigUpdated("TIMEOUT_DEFAULT", timeoutConfig.defaultTimeout, defaultTimeout, msg.sender);
    }

    /**
     * @notice Set chain-specific timeout
     * @param chainId Chain ID
     * @param timeout Timeout for this chain
     */
    function setChainTimeout(
        uint32 chainId,
        uint256 timeout
    ) external onlyRole(ADMIN_ROLE) {
        require(timeout >= 10 minutes, "Timeout too low");
        require(timeout <= 7 days, "Timeout too high");
        
        timeoutConfig.chainSpecificTimeouts[chainId] = timeout;
        
        emit RecoveryConfigUpdated("CHAIN_TIMEOUT", 0, timeout, msg.sender);
    }

    /**
     * @notice Toggle automatic retry functionality
     * @param enabled Whether auto-retry should be enabled
     */
    function setAutoRetryEnabled(bool enabled) external onlyRole(ADMIN_ROLE) {
        autoRetryEnabled = enabled;
        emit RecoveryConfigUpdated("AUTO_RETRY_ENABLED", autoRetryEnabled ? 1 : 0, enabled ? 1 : 0, msg.sender);
    }

    /**
     * @notice Toggle admin recovery functionality
     * @param enabled Whether admin recovery should be enabled
     */
    function setAdminRecoveryEnabled(bool enabled) external onlyRole(ADMIN_ROLE) {
        adminRecoveryEnabled = enabled;
        emit RecoveryConfigUpdated("ADMIN_RECOVERY_ENABLED", adminRecoveryEnabled ? 1 : 0, enabled ? 1 : 0, msg.sender);
    }

    /**
     * @notice Set recovery window duration
     * @param window Time window for admin recovery operations
     */
    function setRecoveryWindow(uint256 window) external onlyRole(ADMIN_ROLE) {
        require(window >= 1 days, "Recovery window too short");
        require(window <= 30 days, "Recovery window too long");
        
        recoveryWindow = window;
        emit RecoveryConfigUpdated("RECOVERY_WINDOW", recoveryWindow, window, msg.sender);
    }

    // ============ View Functions ============

    /**
     * @notice Get message details
     * @param messageId Message ID to query
     * @return pendingMsg Full pending message details
     */
    function getMessage(
        bytes32 messageId
    ) external view returns (ErrorHandling.PendingMessage memory pendingMsg) {
        return pendingMessages[messageId];
    }

    /**
     * @notice Get messages for a specific user
     * @param user User address
     * @return messageIds Array of message IDs for the user
     */
    function getUserMessages(
        address user
    ) external view returns (bytes32[] memory messageIds) {
        return userMessages[user];
    }

    /**
     * @notice Get system statistics
     * @return total Total messages processed
     * @return completed Successfully completed messages
     * @return failed Failed messages
     * @return timeout Timed out messages
     * @return recovered Admin recovered messages
     * @return pending Pending messages
     */
    function getSystemStats() external view returns (
        uint256 total,
        uint256 completed,
        uint256 failed,
        uint256 timeout,
        uint256 recovered,
        uint256 pending
    ) {
        return (
            totalMessages,
            completedMessages,
            failedMessages,
            timeoutMessages,
            recoveredMessages,
            totalMessages - completedMessages - failedMessages
        );
    }

    /**
     * @notice Get current retry configuration
     * @return config Current retry configuration
     */
    function getRetryConfig() external view returns (ErrorHandling.RetryConfig memory config) {
        return retryConfig;
    }

    /**
     * @notice Get timeout configuration for a chain
     * @param chainId Chain ID to query
     * @return timeout Timeout configuration for the chain
     */
    function getChainTimeout(uint32 chainId) external view returns (uint256 timeout) {
        return ErrorHandling.calculateMessageTimeout(chainId, timeoutConfig);
    }

    /**
     * @notice Check if message needs attention (timeout, retry, etc.)
     * @param messageId Message ID to check
     * @return needsAttention Whether message needs attention
     * @return reason Reason why attention is needed
     */
    function checkMessageHealth(
        bytes32 messageId
    ) external view onlyValidMessage(messageId) returns (bool needsAttention, string memory reason) {
        ErrorHandling.PendingMessage storage pendingMsg = pendingMessages[messageId];
        
        (bool hasTimedOut, string memory timeoutReason) = ErrorHandling.isMessageTimedOut(pendingMsg);
        if (hasTimedOut) {
            return (true, string(abi.encodePacked("Timeout: ", timeoutReason)));
        }
        
        if (ErrorHandling.isReadyForRetry(pendingMsg)) {
            return (true, "Ready for retry");
        }
        
        if (pendingMsg.status == ErrorHandling.MessageStatus.FAILED) {
            return (true, string(abi.encodePacked("Failed: ", pendingMsg.lastError)));
        }
        
        return (false, "Message healthy");
    }

    // ============ Emergency Functions ============

    /**
     * @notice Pause the contract (emergency use)
     */
    function pause() external onlyRole(ADMIN_ROLE) {
        _pause();
    }

    /**
     * @notice Unpause the contract
     */
    function unpause() external onlyRole(ADMIN_ROLE) {
        _unpause();
    }

    /**
     * @notice Emergency function to clear stuck messages
     * @param messageIds Messages to clear
     */
    function emergencyClearMessages(
        bytes32[] calldata messageIds
    ) external onlyRole(DEFAULT_ADMIN_ROLE) {
        for (uint256 i = 0; i < messageIds.length; i++) {
            delete pendingMessages[messageIds[i]];
        }
    }
}