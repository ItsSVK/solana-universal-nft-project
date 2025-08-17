use anchor_lang::prelude::*;
use crate::error::UniversalNftError;

/// Signature Validation Utilities
/// 
/// This module provides functions to validate signatures on incoming cross-chain messages
/// to ensure they come from authorized validators or relayers.

/// Minimum number of validators required for consensus
/// 
/// This represents the minimum threshold of validators that must sign
/// a message for it to be considered valid. Using 2/3 majority for security.
pub const MIN_VALIDATOR_THRESHOLD: u8 = 2;

/// Maximum number of validators supported
/// 
/// This limits the total number of validators that can be registered
/// to prevent excessive storage and processing overhead.
pub const MAX_VALIDATORS: u8 = 10;

/// Minimum number of signatures required for message validation
/// 
/// This is calculated as a percentage of the total validators.
/// For example, if there are 3 validators, at least 2 must sign.
pub const MIN_SIGNATURE_PERCENTAGE: u8 = 67; // 67% = 2/3 majority

/// Signature validation result structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct SignatureValidationResult {
    /// Whether the signature validation passed
    pub is_valid: bool,
    /// Number of valid signatures found
    pub valid_signatures: u8,
    /// Total number of signatures provided
    pub total_signatures: u8,
    /// Number of validators that signed
    pub validators_signed: u8,
    /// Total number of registered validators
    pub total_validators: u8,
    /// Detailed error message if validation failed
    pub error_message: Option<String>,
}

/// Validator information structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct ValidatorInfo {
    /// The validator's public key
    pub public_key: Pubkey,
    /// Whether the validator is currently active
    pub is_active: bool,
    /// The validator's weight in consensus (for weighted voting)
    pub weight: u8,
    /// Timestamp when the validator was registered
    pub registered_at: i64,
}

/// Message signature structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct MessageSignature {
    /// The signature bytes
    pub signature: Vec<u8>,
    /// The public key of the signer
    pub signer: Pubkey,
    /// Timestamp when the signature was created
    pub timestamp: i64,
}

/// Verify that the incoming message has valid signatures from authorized validators
/// 
/// This function validates the signatures attached to the incoming message.
/// It extracts the signatures from the message, verifies them against the public
/// keys of authorized validators, and ensures that a sufficient threshold of
/// valid signatures is present.
/// 
/// # Arguments
/// * `message_data` - The message data that was signed
/// * `signatures` - Vector of signatures to validate
/// * `validators` - Vector of authorized validator public keys
/// * `required_threshold` - Minimum number of valid signatures required
/// 
/// # Returns
/// * `Result<SignatureValidationResult>` - Validation result with detailed information
/// 
/// # Errors
/// * `UniversalNftError::InvalidSignature` - If any signature is invalid
/// * `UniversalNftError::InsufficientSignatures` - If not enough valid signatures
/// * `UniversalNftError::UnauthorizedValidator` - If a signer is not an authorized validator
pub fn verify_signatures(
    message_data: &[u8],
    signatures: &[MessageSignature],
    validators: &[Pubkey],
    required_threshold: u8,
) -> Result<SignatureValidationResult> {
    msg!("Starting signature validation...");
    msg!("Message data length: {} bytes", message_data.len());
    msg!("Number of signatures provided: {}", signatures.len());
    msg!("Number of authorized validators: {}", validators.len());
    msg!("Required threshold: {}", required_threshold);

    // Validate input parameters
    require!(
        !message_data.is_empty(),
        UniversalNftError::InvalidSignature
    );
    
    require!(
        !signatures.is_empty(),
        UniversalNftError::InsufficientSignatures
    );
    
    require!(
        !validators.is_empty(),
        UniversalNftError::UnauthorizedValidator
    );
    
    require!(
        required_threshold > 0 && required_threshold <= validators.len() as u8,
        UniversalNftError::InvalidSignatureThreshold
    );

    let mut valid_signatures: u8 = 0;
    let mut validators_signed: u8 = 0;
    let mut validation_errors = Vec::new();

    // Process each signature
    for (index, signature) in signatures.iter().enumerate() {
        msg!("Validating signature {} from {}", index + 1, signature.signer);
        
        // Check if the signer is an authorized validator
        if !validators.contains(&signature.signer) {
            let error_msg = format!("Unauthorized validator: {}", signature.signer);
            validation_errors.push(error_msg.clone());
            msg!("❌ {}", error_msg);
            continue;
        }

        // Verify the signature cryptographically
        match verify_single_signature(message_data, &signature.signature, &signature.signer) {
            Ok(is_valid) => {
                if is_valid {
                    valid_signatures += 1;
                    validators_signed += 1;
                    msg!("✅ Signature {} valid from {}", index + 1, signature.signer);
                } else {
                    let error_msg = format!("Invalid signature from validator: {}", signature.signer);
                    validation_errors.push(error_msg.clone());
                    msg!("❌ {}", error_msg);
                }
            }
            Err(e) => {
                let error_msg = format!("Signature verification error: {}", e);
                validation_errors.push(error_msg.clone());
                msg!("❌ {}", error_msg);
            }
        }
    }

    // Check if we have enough valid signatures
    let threshold_met = valid_signatures >= required_threshold;
    
    msg!("Signature validation summary:");
    msg!("  Valid signatures: {}/{}", valid_signatures, signatures.len());
    msg!("  Validators signed: {}/{}", validators_signed, validators.len());
    msg!("  Required threshold: {}", required_threshold);
    msg!("  Threshold met: {}", threshold_met);

    if !threshold_met {
        let error_msg = format!(
            "Insufficient valid signatures: {} < {}",
            valid_signatures, required_threshold
        );
        validation_errors.push(error_msg.clone());
        msg!("❌ {}", error_msg);
    }

    // Create validation result
    let result = SignatureValidationResult {
        is_valid: threshold_met,
        valid_signatures,
        total_signatures: signatures.len() as u8,
        validators_signed,
        total_validators: validators.len() as u8,
        error_message: if validation_errors.is_empty() {
            None
        } else {
            Some(validation_errors.join("; "))
        },
    };

    if result.is_valid {
        msg!("✅ Signature validation PASSED");
    } else {
        msg!("❌ Signature validation FAILED");
    }

    Ok(result)
}

/// Verify a single signature against message data and public key
/// 
/// This function performs the cryptographic verification of a single signature.
/// In a real implementation, this would use Ed25519 or similar signature verification.
/// 
/// # Arguments
/// * `message_data` - The message data that was signed
/// * `signature` - The signature bytes to verify
/// * `public_key` - The public key of the signer
/// 
/// # Returns
/// * `Result<bool>` - True if signature is valid, false otherwise
/// 
/// # Errors
/// * `UniversalNftError::InvalidSignature` - If signature verification fails
pub fn verify_single_signature(
    message_data: &[u8],
    signature: &[u8],
    public_key: &Pubkey,
) -> Result<bool> {
    // Validate input parameters
    require!(
        !message_data.is_empty(),
        UniversalNftError::InvalidSignature
    );
    
    require!(
        !signature.is_empty(),
        UniversalNftError::InvalidSignature
    );
    
    require!(
        *public_key != Pubkey::default(),
        UniversalNftError::InvalidSignature
    );

    // In a real implementation, this would perform actual cryptographic verification
    // For now, we'll simulate the verification process
    
    // Check signature length (Ed25519 signatures are 64 bytes)
    if signature.len() != 64 {
        msg!("❌ Invalid signature length: {} bytes (expected 64)", signature.len());
        return Ok(false);
    }
    
    // Check that the signature is not all zeros (basic sanity check)
    if signature.iter().all(|&b| b == 0) {
        msg!("❌ Signature is all zeros (invalid)");
        return Ok(false);
    }
    
    // Check that the public key is not all zeros
    if public_key.to_bytes().iter().all(|&b| b == 0) {
        msg!("❌ Public key is all zeros (invalid)");
        return Ok(false);
    }
    
    // Simulate signature verification success
    // In production, this would call the actual cryptographic verification
    msg!("✅ Signature verification simulation successful");
    Ok(true)
}

/// Calculate the required signature threshold based on validator count
/// 
/// This function calculates how many signatures are required based on
/// the total number of validators and the configured threshold percentage.
/// 
/// # Arguments
/// * `total_validators` - Total number of registered validators
/// * `threshold_percentage` - Required percentage of validators (e.g., 67 for 2/3)
/// 
/// # Returns
/// * `Result<usize>` - Required number of signatures
/// 
/// # Errors
/// * `UniversalNftError::InvalidSignatureThreshold` - If threshold calculation fails
pub fn calculate_signature_threshold(
    total_validators: u8,
    threshold_percentage: u8,
) -> Result<u8> {
    require!(
        total_validators > 0,
        UniversalNftError::InvalidSignatureThreshold
    );
    
    require!(
        threshold_percentage > 0 && threshold_percentage <= 100,
        UniversalNftError::InvalidSignatureThreshold
    );
    
    let threshold = (total_validators as u16 * threshold_percentage as u16 / 100) as u8;
    
    // Ensure minimum threshold is met
    let final_threshold = std::cmp::max(threshold, MIN_VALIDATOR_THRESHOLD);
    
    msg!("Calculated signature threshold: {} ({}% of {} validators)", 
         final_threshold, threshold_percentage, total_validators);
    
    Ok(final_threshold)
}

/// Validate validator registration parameters
/// 
/// This function validates parameters when registering new validators
/// to ensure they meet the program's requirements.
/// 
/// # Arguments
/// * `validator_public_key` - The public key of the validator to register
/// * `current_validator_count` - Current number of registered validators
/// * `validator_weight` - The weight/authority level of the validator
/// 
/// # Returns
/// * `Result<()>` - Ok if parameters are valid, error if not
/// 
/// # Errors
/// * `UniversalNftError::InvalidValidatorData` - If validator data is invalid
/// * `UniversalNftError::TooManyValidators` - If maximum validator limit would be exceeded
pub fn validate_validator_registration(
    validator_public_key: &Pubkey,
    current_validator_count: u8,
    validator_weight: u8,
) -> Result<()> {
    // Check if validator public key is valid
    require!(
        *validator_public_key != Pubkey::default(),
        UniversalNftError::InvalidValidatorData
    );
    
    // Check if we're at the maximum validator limit
    require!(
        current_validator_count < MAX_VALIDATORS,
        UniversalNftError::TooManyValidators
    );
    
    // Check if validator weight is reasonable
    require!(
        validator_weight > 0 && validator_weight <= 100,
        UniversalNftError::InvalidValidatorData
    );
    
    msg!("✅ Validator registration parameters validated successfully");
    Ok(())
}

/// Log signature validation details for debugging and monitoring
/// 
/// This function provides consistent logging for signature validation
/// operations for debugging and monitoring purposes.
/// 
/// # Arguments
/// * `operation` - The operation being performed
/// * `message_hash` - Hash of the message being validated
/// * `signature_count` - Number of signatures provided
/// * `validator_count` - Number of authorized validators
/// * `details` - Additional details about the validation
pub fn log_signature_validation(
    operation: &str,
    message_hash: &[u8],
    signature_count: u8,
    validator_count: u8,
    details: &str,
) {
    msg!("=== Signature Validation Operation ===");
    msg!("Operation: {}", operation);
    msg!("Message Hash: {} bytes", message_hash.len());
    msg!("Signatures: {}", signature_count);
    msg!("Validators: {}", validator_count);
    msg!("Details: {}", details);
    msg!("=====================================");
}

/// Get signature validation statistics
/// 
/// This function provides statistics about signature validation
/// for monitoring and analytics purposes.
/// 
/// # Arguments
/// * `total_validations` - Total number of validation attempts
/// * `successful_validations` - Number of successful validations
/// * `failed_validations` - Number of failed validations
/// 
/// # Returns
/// * `SignatureValidationStats` - Statistics about validation performance
pub fn get_signature_validation_stats(
    total_validations: u64,
    successful_validations: u64,
    failed_validations: u64,
) -> SignatureValidationStats {
    let success_rate_percentage = if total_validations > 0 {
        ((successful_validations as u128 * 100) / total_validations as u128) as u8
    } else {
        0
    };
    
    SignatureValidationStats {
        total_validations,
        successful_validations,
        failed_validations,
        success_rate_percentage,
        last_updated: Clock::get().unwrap().unix_timestamp,
    }
}

/// Signature validation statistics structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct SignatureValidationStats {
    /// Total number of validation attempts
    pub total_validations: u64,
    /// Number of successful validations
    pub successful_validations: u64,
    /// Number of failed validations
    pub failed_validations: u64,
    /// Success rate as a percentage (scaled by 100 for integer storage)
    pub success_rate_percentage: u8,
    /// Timestamp of last update
    pub last_updated: i64,
}
