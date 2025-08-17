/// Gateway Error Handling and Documentation
/// 
/// This module provides comprehensive error handling for Gateway-related operations
/// and documents the Gateway integration patterns.

use anchor_lang::prelude::*;
use crate::error::UniversalNftError;

/// Gateway operation result with detailed error information
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct GatewayOperationResult {
    /// Whether the operation was successful
    pub success: bool,
    /// Gateway operation type
    pub operation_type: GatewayOperationType,
    /// Detailed error message if operation failed
    pub error_message: Option<String>,
    /// Gateway-specific error code
    pub gateway_error_code: Option<u32>,
    /// Timestamp of the operation
    pub timestamp: i64,
}

/// Types of Gateway operations
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum GatewayOperationType {
    /// Incoming cross-chain message
    IncomingMessage,
    /// Outgoing cross-chain message
    OutgoingMessage,
    /// Gateway caller validation
    CallerValidation,
    /// Replay protection check
    ReplayProtection,
    /// Message processing
    MessageProcessing,
}

impl GatewayOperationType {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            Self::IncomingMessage => "Incoming cross-chain message",
            Self::OutgoingMessage => "Outgoing cross-chain message",
            Self::CallerValidation => "Gateway caller validation",
            Self::ReplayProtection => "Replay protection check",
            Self::MessageProcessing => "Message processing",
        }
    }
}

/// Handle Gateway-specific errors with detailed logging
pub fn handle_gateway_error(
    operation_type: GatewayOperationType,
    error: &UniversalNftError,
    context: &str,
) -> GatewayOperationResult {
    let error_message = match error {
        UniversalNftError::UnauthorizedGateway => {
            "Gateway caller is not authorized to perform this operation"
        }
        UniversalNftError::GatewayNotActive => {
            "Gateway integration is not currently active"
        }
        UniversalNftError::InvalidGatewayData => {
            "Gateway data is invalid or malformed"
        }
        UniversalNftError::GatewayCallFailed => {
            "Gateway CPI call failed"
        }
        UniversalNftError::MessageAlreadyProcessed => {
            "Cross-chain message has already been processed"
        }
        UniversalNftError::InvalidCrossChainMessage => {
            "Cross-chain message validation failed"
        }
        _ => "Unknown Gateway-related error",
    };

    let gateway_error_code = match error {
        UniversalNftError::UnauthorizedGateway => Some(1001),
        UniversalNftError::GatewayNotActive => Some(1002),
        UniversalNftError::InvalidGatewayData => Some(1003),
        UniversalNftError::GatewayCallFailed => Some(1004),
        UniversalNftError::MessageAlreadyProcessed => Some(1005),
        UniversalNftError::InvalidCrossChainMessage => Some(1006),
        _ => Some(9999),
    };

    msg!("🚨 Gateway Error in {}: {}", operation_type.description(), error_message);
    msg!("   Context: {}", context);
    msg!("   Error Code: {:?}", gateway_error_code);
    msg!("   Error Type: {:?}", error);

    GatewayOperationResult {
        success: false,
        operation_type,
        error_message: Some(error_message.to_string()),
        gateway_error_code,
        timestamp: Clock::get().unwrap().unix_timestamp,
    }
}

/// Log Gateway operation success
pub fn log_gateway_success(
    operation_type: GatewayOperationType,
    context: &str,
    details: &str,
) -> GatewayOperationResult {
    msg!("✅ Gateway {} completed successfully", operation_type.description());
    msg!("   Context: {}", context);
    msg!("   Details: {}", details);

    GatewayOperationResult {
        success: true,
        operation_type,
        error_message: None,
        gateway_error_code: None,
        timestamp: Clock::get().unwrap().unix_timestamp,
    }
}

/// Get Gateway operation statistics
pub fn get_gateway_operation_stats() -> GatewayOperationStats {
    GatewayOperationStats {
        total_operations: 0,
        successful_operations: 0,
        failed_operations: 0,
        last_operation_timestamp: Clock::get().unwrap().unix_timestamp,
    }
}

/// Gateway operation statistics
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct GatewayOperationStats {
    /// Total number of Gateway operations
    pub total_operations: u64,
    /// Number of successful operations
    pub successful_operations: u64,
    /// Number of failed operations
    pub failed_operations: u64,
    /// Timestamp of last operation
    pub last_operation_timestamp: i64,
}

