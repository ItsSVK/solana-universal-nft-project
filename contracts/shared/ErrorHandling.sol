// SPDX-License-Identifier: MIT
pragma solidity 0.8.26;

import "@openzeppelin/contracts/access/Ownable.sol";
import "@openzeppelin/contracts/utils/ReentrancyGuard.sol";
import "../../shared/CrossChainMessage.sol";

/**
 * @title ErrorHandling
 * @notice Comprehensive error handling and recovery system for cross-chain NFT transfers
 * @dev Provides timeout, retry, and admin recovery functionalities with extensive logging
 */
library ErrorHandling {
    using CrossChainMessage for CrossChainMessage.NFTTransferMessage;

    // ============ Constants ============
    
    uint256 public constant DEFAULT_MESSAGE_TIMEOUT = 1 hours;
    uint256 public constant EXTENDED_MESSAGE_TIMEOUT = 24 hours;
    uint256 public constant MAX_RETRY_ATTEMPTS = 5;
    uint256 public constant MIN_RETRY_DELAY = 5 minutes;
    uint256 public constant MAX_RETRY_DELAY = 4 hours;
    uint256 public constant RETRY_BACKOFF_MULTIPLIER = 2;

    // ============ Enums ============

    enum MessageStatus {
        PENDING,        // Message created but not processed
        PROCESSING,     // Message currently being processed
        COMPLETED,      // Message successfully processed
        TIMEOUT,        // Message has timed out
        FAILED,         // Message failed after all retries
        ADMIN_RESOLVED, // Message resolved by admin intervention
        CANCELLED       // Message cancelled by user or system
    }

    enum RecoveryAction {
        RETRY,          // Retry the message
        FORCE_COMPLETE, // Force completion (admin only)
        ROLLBACK,       // Rollback the transaction
        MANUAL_RESOLVE, // Mark as manually resolved
        CANCEL          // Cancel the transaction
    }

    // ============ Structs ============

    struct PendingMessage {
        bytes32 messageId;
        CrossChainMessage.NFTTransferMessage message;
        uint256 createdAt;
        uint256 timeout;
        uint256 retryCount;
        uint256 nextRetryAt;
        MessageStatus status;
        string lastError;
        address originalSender;
        bool isRecoverable;
        bytes32 recoveryHash; // For admin verification
    }

    struct RetryConfig {
        uint256 maxRetries;
        uint256 baseDelay;
        uint256 maxDelay;
        uint256 backoffMultiplier;
        bool exponentialBackoff;
    }

    struct TimeoutConfig {
        uint256 defaultTimeout;
        uint256 extendedTimeout;
        mapping(uint32 => uint256) chainSpecificTimeouts;
        bool useChainSpecificTimeouts;
    }

    // ============ Events ============

    event MessageTimeout(
        bytes32 indexed messageId,
        uint32 indexed destinationChain,
        address indexed sender,
        uint256 timeoutAt,
        string reason
    );

    event MessageRetryScheduled(
        bytes32 indexed messageId,
        uint256 indexed retryAttempt,
        uint256 nextRetryAt,
        uint256 delay,
        string reason
    );

    event MessageRetryFailed(
        bytes32 indexed messageId,
        uint256 indexed retryAttempt,
        string error,
        bool willRetryAgain
    );

    event MessageRecovered(
        bytes32 indexed messageId,
        RecoveryAction indexed action,
        address indexed admin,
        string details
    );

    event MessageStatusChanged(
        bytes32 indexed messageId,
        MessageStatus indexed oldStatus,
        MessageStatus indexed newStatus,
        string reason
    );

    event ErrorLogged(
        bytes32 indexed messageId,
        string indexed errorType,
        string errorMessage,
        bytes errorData
    );

    // ============ Timeout Functions ============

    /**
     * @notice Check if a message has timed out
     * @param pendingMsg The pending message to check
     * @return hasTimedOut Whether the message has timed out
     * @return timeoutReason Reason for timeout
     */
    function isMessageTimedOut(
        PendingMessage storage pendingMsg
    ) internal view returns (bool hasTimedOut, string memory timeoutReason) {
        if (pendingMsg.status == MessageStatus.COMPLETED || 
            pendingMsg.status == MessageStatus.ADMIN_RESOLVED ||
            pendingMsg.status == MessageStatus.CANCELLED) {
            return (false, "");
        }

        uint256 currentTime = block.timestamp;
        uint256 messageAge = currentTime - pendingMsg.createdAt;

        if (messageAge > pendingMsg.timeout) {
            if (pendingMsg.retryCount >= MAX_RETRY_ATTEMPTS) {
                return (true, "Maximum retries exceeded and timeout reached");
            } else {
                return (true, "Standard timeout reached");
            }
        }

        return (false, "");
    }

    /**
     * @notice Calculate timeout for a message based on destination chain
     * @param destinationChain The destination chain ID
     * @param config Timeout configuration
     * @return timeout Calculated timeout in seconds
     */
    function calculateMessageTimeout(
        uint32 destinationChain,
        TimeoutConfig storage config
    ) internal view returns (uint256 timeout) {
        if (config.useChainSpecificTimeouts && config.chainSpecificTimeouts[destinationChain] > 0) {
            return config.chainSpecificTimeouts[destinationChain];
        }
        
        // Default timeout based on chain characteristics
        if (destinationChain == 900) { // Solana
            return config.extendedTimeout; // Solana might need more time
        } else {
            return config.defaultTimeout;
        }
    }

    /**
     * @notice Handle message timeout
     * @param pendingMsg The timed out message
     * @param retryConfig Retry configuration
     * @return shouldRetry Whether to schedule a retry
     * @return retryDelay Delay before retry (if applicable)
     */
    function handleMessageTimeout(
        PendingMessage storage pendingMsg,
        RetryConfig storage retryConfig
    ) internal returns (bool shouldRetry, uint256 retryDelay) {
        (bool hasTimedOut, string memory reason) = isMessageTimedOut(pendingMsg);
        
        if (!hasTimedOut) {
            return (false, 0);
        }

        emit MessageTimeout(
            pendingMsg.messageId,
            pendingMsg.message.destinationChain,
            pendingMsg.originalSender,
            block.timestamp,
            reason
        );

        // Update status
        MessageStatus oldStatus = pendingMsg.status;
        pendingMsg.status = MessageStatus.TIMEOUT;
        
        emit MessageStatusChanged(
            pendingMsg.messageId,
            oldStatus,
            MessageStatus.TIMEOUT,
            reason
        );

        // Check if we should retry
        if (pendingMsg.retryCount < retryConfig.maxRetries && pendingMsg.isRecoverable) {
            retryDelay = calculateRetryDelay(pendingMsg.retryCount, retryConfig);
            return (true, retryDelay);
        } else {
            // Mark as failed
            pendingMsg.status = MessageStatus.FAILED;
            emit MessageStatusChanged(
                pendingMsg.messageId,
                MessageStatus.TIMEOUT,
                MessageStatus.FAILED,
                "Maximum retries exceeded"
            );
            return (false, 0);
        }
    }

    // ============ Retry Functions ============

    /**
     * @notice Calculate retry delay with exponential backoff
     * @param retryAttempt Current retry attempt number
     * @param config Retry configuration
     * @return delay Delay in seconds before next retry
     */
    function calculateRetryDelay(
        uint256 retryAttempt,
        RetryConfig storage config
    ) internal view returns (uint256 delay) {
        if (config.exponentialBackoff) {
            delay = config.baseDelay * (config.backoffMultiplier ** retryAttempt);
        } else {
            delay = config.baseDelay;
        }
        
        // Cap at maximum delay
        if (delay > config.maxDelay) {
            delay = config.maxDelay;
        }
        
        // Ensure minimum delay
        if (delay < MIN_RETRY_DELAY) {
            delay = MIN_RETRY_DELAY;
        }
        
        return delay;
    }

    /**
     * @notice Schedule message retry
     * @param pendingMsg The message to retry
     * @param config Retry configuration
     * @param reason Reason for retry
     */
    function scheduleRetry(
        PendingMessage storage pendingMsg,
        RetryConfig storage config,
        string memory reason
    ) internal {
        require(pendingMsg.retryCount < config.maxRetries, "Maximum retries exceeded");
        require(pendingMsg.isRecoverable, "Message not recoverable");

        uint256 delay = calculateRetryDelay(pendingMsg.retryCount, config);
        pendingMsg.nextRetryAt = block.timestamp + delay;
        pendingMsg.retryCount++;

        MessageStatus oldStatus = pendingMsg.status;
        pendingMsg.status = MessageStatus.PENDING;

        emit MessageRetryScheduled(
            pendingMsg.messageId,
            pendingMsg.retryCount,
            pendingMsg.nextRetryAt,
            delay,
            reason
        );

        emit MessageStatusChanged(
            pendingMsg.messageId,
            oldStatus,
            MessageStatus.PENDING,
            string(abi.encodePacked("Retry scheduled: ", reason))
        );
    }

    /**
     * @notice Check if message is ready for retry
     * @param pendingMsg The pending message
     * @return isReady Whether the message is ready for retry
     */
    function isReadyForRetry(
        PendingMessage storage pendingMsg
    ) internal view returns (bool isReady) {
        return pendingMsg.status == MessageStatus.PENDING &&
               pendingMsg.retryCount > 0 &&
               block.timestamp >= pendingMsg.nextRetryAt;
    }

    /**
     * @notice Handle retry failure
     * @param pendingMsg The message that failed retry
     * @param error Error message
     * @param config Retry configuration
     */
    function handleRetryFailure(
        PendingMessage storage pendingMsg,
        string memory error,
        RetryConfig storage config
    ) internal {
        pendingMsg.lastError = error;
        
        bool willRetryAgain = pendingMsg.retryCount < config.maxRetries && pendingMsg.isRecoverable;
        
        emit MessageRetryFailed(
            pendingMsg.messageId,
            pendingMsg.retryCount,
            error,
            willRetryAgain
        );

        if (willRetryAgain) {
            scheduleRetry(pendingMsg, config, error);
        } else {
            MessageStatus oldStatus = pendingMsg.status;
            pendingMsg.status = MessageStatus.FAILED;
            
            emit MessageStatusChanged(
                pendingMsg.messageId,
                oldStatus,
                MessageStatus.FAILED,
                string(abi.encodePacked("Final retry failed: ", error))
            );
        }
    }

    // ============ Recovery Functions ============

    /**
     * @notice Generate recovery hash for admin operations
     * @param messageId Message ID
     * @param action Recovery action
     * @param admin Admin address
     * @return recoveryHash Hash for verification
     */
    function generateRecoveryHash(
        bytes32 messageId,
        RecoveryAction action,
        address admin
    ) internal view returns (bytes32 recoveryHash) {
        return keccak256(abi.encodePacked(
            messageId,
            uint256(action),
            admin,
            block.timestamp,
            block.chainid
        ));
    }

    /**
     * @notice Verify recovery operation
     * @param pendingMsg The message to recover
     * @param action Recovery action
     * @param admin Admin address performing recovery
     * @param providedHash Recovery hash provided by admin
     * @return isValid Whether the recovery is valid
     */
    function verifyRecovery(
        PendingMessage storage pendingMsg,
        RecoveryAction action,
        address admin,
        bytes32 providedHash
    ) internal view returns (bool isValid) {
        // Check if message is in a recoverable state
        if (pendingMsg.status == MessageStatus.COMPLETED ||
            pendingMsg.status == MessageStatus.ADMIN_RESOLVED ||
            pendingMsg.status == MessageStatus.CANCELLED) {
            return false;
        }

        // For high-risk actions, require hash verification
        if (action == RecoveryAction.FORCE_COMPLETE || action == RecoveryAction.ROLLBACK) {
            bytes32 expectedHash = generateRecoveryHash(pendingMsg.messageId, action, admin);
            return providedHash == expectedHash;
        }

        return true; // Lower risk actions don't require hash verification
    }

    /**
     * @notice Execute admin recovery action
     * @param pendingMsg The message to recover
     * @param action Recovery action to perform
     * @param admin Admin performing the action
     * @param details Additional details about the recovery
     */
    function executeRecovery(
        PendingMessage storage pendingMsg,
        RecoveryAction action,
        address admin,
        string memory details
    ) internal {
        MessageStatus oldStatus = pendingMsg.status;
        MessageStatus newStatus;

        if (action == RecoveryAction.FORCE_COMPLETE) {
            newStatus = MessageStatus.ADMIN_RESOLVED;
            pendingMsg.isRecoverable = false;
        } else if (action == RecoveryAction.CANCEL) {
            newStatus = MessageStatus.CANCELLED;
            pendingMsg.isRecoverable = false;
        } else if (action == RecoveryAction.RETRY) {
            newStatus = MessageStatus.PENDING;
            pendingMsg.nextRetryAt = block.timestamp + MIN_RETRY_DELAY;
            pendingMsg.lastError = "";
        } else if (action == RecoveryAction.ROLLBACK) {
            newStatus = MessageStatus.CANCELLED;
            pendingMsg.isRecoverable = false;
        } else if (action == RecoveryAction.MANUAL_RESOLVE) {
            newStatus = MessageStatus.ADMIN_RESOLVED;
            pendingMsg.isRecoverable = false;
        }

        pendingMsg.status = newStatus;
        pendingMsg.recoveryHash = generateRecoveryHash(pendingMsg.messageId, action, admin);

        emit MessageRecovered(
            pendingMsg.messageId,
            action,
            admin,
            details
        );

        emit MessageStatusChanged(
            pendingMsg.messageId,
            oldStatus,
            newStatus,
            string(abi.encodePacked("Admin recovery: ", details))
        );
    }

    // ============ Logging Functions ============

    /**
     * @notice Log error with structured data
     * @param messageId Associated message ID
     * @param errorType Category of error
     * @param errorMessage Human-readable error message
     * @param errorData Additional structured error data
     */
    function logError(
        bytes32 messageId,
        string memory errorType,
        string memory errorMessage,
        bytes memory errorData
    ) internal {
        emit ErrorLogged(messageId, errorType, errorMessage, errorData);
    }

    /**
     * @notice Log structured error for common scenarios
     * @param messageId Associated message ID
     * @param errorType Category of error
     * @param errorMessage Human-readable error message
     */
    function logSimpleError(
        bytes32 messageId,
        string memory errorType,
        string memory errorMessage
    ) internal {
        emit ErrorLogged(messageId, errorType, errorMessage, "");
    }

    // ============ Utility Functions ============

    /**
     * @notice Create new pending message
     * @param message Cross-chain message
     * @param sender Original sender
     * @param timeout Message timeout
     * @return pendingMsg New pending message struct
     */
    function createPendingMessage(
        CrossChainMessage.NFTTransferMessage memory message,
        address sender,
        uint256 timeout
    ) internal view returns (PendingMessage memory pendingMsg) {
        return PendingMessage({
            messageId: message.messageId,
            message: message,
            createdAt: block.timestamp,
            timeout: timeout,
            retryCount: 0,
            nextRetryAt: 0,
            status: MessageStatus.PENDING,
            lastError: "",
            originalSender: sender,
            isRecoverable: true,
            recoveryHash: bytes32(0)
        });
    }

    /**
     * @notice Check if message status is final (cannot be changed)
     * @param status Message status to check
     * @return isFinal Whether the status is final
     */
    function isStatusFinal(MessageStatus status) internal pure returns (bool isFinal) {
        return status == MessageStatus.COMPLETED ||
               status == MessageStatus.ADMIN_RESOLVED ||
               status == MessageStatus.CANCELLED;
    }

    /**
     * @notice Get human-readable status name
     * @param status Message status
     * @return statusName Human-readable status name
     */
    function getStatusName(MessageStatus status) internal pure returns (string memory statusName) {
        if (status == MessageStatus.PENDING) return "PENDING";
        if (status == MessageStatus.PROCESSING) return "PROCESSING";
        if (status == MessageStatus.COMPLETED) return "COMPLETED";
        if (status == MessageStatus.TIMEOUT) return "TIMEOUT";
        if (status == MessageStatus.FAILED) return "FAILED";
        if (status == MessageStatus.ADMIN_RESOLVED) return "ADMIN_RESOLVED";
        if (status == MessageStatus.CANCELLED) return "CANCELLED";
        return "UNKNOWN";
    }
}