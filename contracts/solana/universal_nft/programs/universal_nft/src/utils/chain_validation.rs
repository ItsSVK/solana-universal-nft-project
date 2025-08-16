use anchor_lang::prelude::*;
use crate::constants::chains::*;
use crate::error::UniversalNftError;

/// Verify that a chain ID is supported
/// 
/// This function checks if the incoming message's chain ID is in our
/// list of supported chains. Only messages from supported chains
/// will be processed.
/// 
/// # Arguments
/// * `chain_id` - The chain ID from the incoming cross-chain message
/// 
/// # Returns
/// * `Result<()>` - Ok if chain is supported, error if not
/// 
/// # Errors
/// * `UniversalNftError::UnsupportedChainId` - If the chain ID is not supported
pub fn verify_chain_id(chain_id: u8) -> Result<()> {
    require!(
        is_chain_supported(chain_id),
        UniversalNftError::UnsupportedChainId
    );
    
    msg!("Chain ID {} verified as supported", chain_id);
    Ok(())
}

/// Verify that a chain ID is valid (basic format check)
/// 
/// This function performs basic validation on the chain ID format.
/// Currently, we only support u8 chain IDs (0-255).
/// 
/// # Arguments
/// * `chain_id` - The chain ID to validate
/// 
/// # Returns
/// * `Result<()>` - Ok if format is valid, error if not
/// 
/// # Errors
/// * `UniversalNftError::InvalidChainIdFormat` - If the chain ID format is invalid
pub fn validate_chain_id_format(chain_id: u8) -> Result<()> {
    // For now, all u8 values are valid
    // In the future, we might add more specific format requirements
    // For example, reserving certain ranges for specific purposes
    
    // Check if chain ID is in reserved range (0 is typically reserved)
    require!(
        chain_id != 0,
        UniversalNftError::InvalidChainIdFormat
    );
    
    msg!("Chain ID {} format validated", chain_id);
    Ok(())
}

/// Comprehensive chain ID validation
/// 
/// This function combines both format validation and support checking.
/// It's the main entry point for validating incoming chain IDs.
/// 
/// # Arguments
/// * `chain_id` - The chain ID from the incoming cross-chain message
/// 
/// # Returns
/// * `Result<()>` - Ok if chain ID is valid and supported, error if not
/// 
/// # Errors
/// * `UniversalNftError::InvalidChainIdFormat` - If the chain ID format is invalid
/// * `UniversalNftError::UnsupportedChainId` - If the chain ID is not supported
pub fn validate_chain_id(chain_id: u8) -> Result<()> {
    // First validate the format
    validate_chain_id_format(chain_id)?;
    
    // Then check if it's supported
    verify_chain_id(chain_id)?;
    
    msg!("Chain ID {} fully validated and supported", chain_id);
    Ok(())
}

/// Get information about supported chains for debugging/logging
pub fn get_chain_info(chain_id: u8) -> &'static str {
    match chain_id {
        CHAIN_ID_SOLANA => "Solana",
        CHAIN_ID_BASE_SEPOLIA => "Base Sepolia",
        CHAIN_ID_BNB_TESTNET => "BNB Smart Chain Testnet",
        _ => "Unknown Chain",
    }
}

/// Check if a chain ID is a testnet
pub fn is_testnet_chain(chain_id: u8) -> bool {
    matches!(
        chain_id,
        CHAIN_ID_BASE_SEPOLIA | CHAIN_ID_BNB_TESTNET
    )
}

/// Check if a chain ID is a mainnet
pub fn is_mainnet_chain(chain_id: u8) -> bool {
    chain_id == CHAIN_ID_SOLANA
}
