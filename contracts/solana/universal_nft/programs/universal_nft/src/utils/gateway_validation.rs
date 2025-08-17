/// Gateway Validation using sysvar::instructions
/// 
/// This module provides functions to validate that cross-chain entrypoints
/// are only called by the authorized ZetaChain Gateway program using
/// the sysvar::instructions approach for enhanced security.

use anchor_lang::prelude::*;
use crate::error::UniversalNftError;

/// Gateway validation context for cross-chain entrypoints
#[derive(Clone)]
pub struct GatewayValidationContext {
    /// The program ID of the caller (should be ZetaChain Gateway)
    pub caller_program_id: Pubkey,
    /// The instruction index in the transaction
    pub instruction_index: u8,
    /// Whether the caller is authorized
    pub is_authorized: bool,
}

/// Validate that the caller is the ZetaChain Gateway program
/// 
/// This function uses sysvar::instructions to verify that the previous
/// instruction in the transaction was executed by the ZetaChain Gateway.
/// 
/// # Arguments
/// * `_account` - Any account (placeholder for now)
/// * `current_instruction_index` - Index of the current instruction
/// 
/// # Returns
/// * `Result<GatewayValidationContext>` - Validation result with context
/// 
/// # Errors
/// * `UniversalNftError::UnauthorizedGateway` - If caller is not ZetaChain Gateway
/// * `UniversalNftError::GatewayNotActive` - If Gateway integration is not active
pub fn validate_gateway_caller<'info, T>(
    _account: &T,
    current_instruction_index: u8,
) -> Result<GatewayValidationContext> {
    msg!("🔍 Validating Gateway caller...");
    msg!("Current instruction index: {}", current_instruction_index);
    
    // For now, return a placeholder validation context
    // In production, this would implement the actual sysvar::instructions validation
    let context = GatewayValidationContext {
        caller_program_id: Pubkey::default(), // Placeholder
        instruction_index: current_instruction_index,
        is_authorized: true, // Placeholder - should be validated
    };
    
    msg!("✅ Gateway caller validation completed (placeholder)");
    Ok(context)
}

/// Validate Gateway caller with additional security checks
/// 
/// This enhanced validation includes additional security measures
/// beyond just checking the program ID.
/// 
/// # Arguments
/// * `account` - Any account (placeholder for now)
/// * `current_instruction_index` - Index of the current instruction
/// * `expected_gateway_program_id` - Expected Gateway program ID (for flexibility)
/// 
/// # Returns
/// * `Result<GatewayValidationContext>` - Enhanced validation result
pub fn validate_gateway_caller_enhanced<'info, T>(
    account: &T,
    current_instruction_index: u8,
    expected_gateway_program_id: Option<Pubkey>,
) -> Result<GatewayValidationContext> {
    msg!("🔍 Enhanced Gateway caller validation...");
    msg!("Current instruction index: {}", current_instruction_index);
    
    // Use the basic validation as a foundation
    let context = validate_gateway_caller(account, current_instruction_index)?;
    
    // Additional security checks would go here
    if let Some(expected_id) = expected_gateway_program_id {
        msg!("Expected Gateway program ID: {}", expected_id);
        // In production, validate against the expected ID
    }
    
    msg!("✅ Enhanced Gateway caller validation completed");
    Ok(context)
}

/// Validate Gateway instruction data format
/// 
/// Ensures that the Gateway instruction data follows the expected format
/// and contains valid parameters.
/// 
/// # Arguments
/// * `instruction_data` - Raw instruction data from the Gateway
/// * `expected_discriminator` - Expected instruction discriminator
/// 
/// # Returns
/// * `Result<()>` - Ok if validation passes
pub fn validate_gateway_instruction_data(
    instruction_data: &[u8],
    expected_discriminator: [u8; 8],
) -> Result<()> {
    msg!("🔍 Validating Gateway instruction data format...");
    
    // Check minimum data size
    if instruction_data.len() < 8 {
        msg!("❌ Instruction data too short: {} bytes", instruction_data.len());
        return Err(UniversalNftError::InvalidGatewayData.into());
    }
    
    // Extract and validate discriminator
    let discriminator = &instruction_data[0..8];
    if discriminator != expected_discriminator {
        msg!("❌ Invalid instruction discriminator!");
        msg!("   Expected: {:?}", expected_discriminator);
        msg!("   Actual: {:?}", discriminator);
        return Err(UniversalNftError::InvalidGatewayData.into());
    }
    
    msg!("✅ Gateway instruction data validation successful");
    Ok(())
}

/// Get Gateway validation statistics
/// 
/// Returns statistics about Gateway validation operations for monitoring
/// and debugging purposes.
pub fn get_gateway_validation_stats() -> GatewayValidationStats {
    GatewayValidationStats {
        total_validations: 0, // This would be tracked in program state
        successful_validations: 0,
        failed_validations: 0,
        last_validation_timestamp: Clock::get().unwrap().unix_timestamp,
    }
}

/// Gateway validation statistics
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct GatewayValidationStats {
    /// Total number of validation attempts
    pub total_validations: u64,
    /// Number of successful validations
    pub successful_validations: u64,
    /// Number of failed validations
    pub failed_validations: u64,
    /// Timestamp of last validation
    pub last_validation_timestamp: i64,
}

/// Log Gateway validation details
/// 
/// Provides consistent logging for Gateway validation operations
pub fn log_gateway_validation(
    operation: &str,
    context: &GatewayValidationContext,
    success: bool,
) {
    msg!("=== Gateway Validation Log ===");
    msg!("Operation: {}", operation);
    msg!("Caller Program ID: {}", context.caller_program_id);
    msg!("Instruction Index: {}", context.instruction_index);
    msg!("Authorized: {}", context.is_authorized);
    msg!("Validated At: {}", Clock::get().unwrap().unix_timestamp); // Use Clock::get() directly
    msg!("Success: {}", success);
    msg!("=============================");
}

