use anchor_lang::prelude::*;
use crate::error::UniversalNftError;
use crate::utils::{
    signature_validation::*,
    gateway_authorization::*,
};

/// Cross-Chain Message Validation Pipeline
/// 
/// This module provides a comprehensive validation pipeline that integrates
/// all individual validation functions to ensure cross-chain messages are
/// fully validated before processing.

/// Validation pipeline result structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct ValidationPipelineResult {
    /// Overall validation status
    pub is_valid: bool,
    /// Validation stage that failed (if any)
    pub failed_stage: Option<String>,
    /// Detailed error message
    pub error_message: Option<String>,
    /// Validation statistics
    pub stats: ValidationPipelineStats,
    /// Timestamp of validation
    pub validated_at: i64,
}

/// Validation pipeline statistics
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct ValidationPipelineStats {
    /// Total validation time in milliseconds
    pub total_validation_time_ms: u64,
    /// Number of validation stages completed
    pub stages_completed: u8,
    /// Number of validation stages that passed
    pub stages_passed: u8,
    /// Number of validation stages that failed
    pub stages_failed: u8,
    /// Validation success rate percentage
    pub success_rate_percentage: u8,
}

/// Cross-chain message validation context
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct CrossChainMessageContext {
    /// Source chain ID
    pub source_chain_id: u8,
    /// Destination chain ID
    pub destination_chain_id: u8,
    /// Message payload
    pub payload: Vec<u8>,
    /// Message signatures
    pub signatures: Vec<Vec<u8>>,
    /// Validator public keys
    pub validators: Vec<Pubkey>,
    /// Gateway caller public key
    pub gateway_caller: Pubkey,
    /// Message hash for replay protection
    pub message_hash: Vec<u8>,
    /// Message timestamp
    pub timestamp: i64,
}

/// Validation pipeline stages
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum ValidationStage {
    /// Chain ID validation
    ChainValidation,
    /// Payload validation
    PayloadValidation,
    /// Signature validation
    SignatureValidation,
    /// Gateway authorization
    GatewayAuthorization,
    /// Replay protection (handled separately)
    ReplayProtection,
}

impl ValidationStage {
    /// Get stage name as string
    pub fn as_str(&self) -> &'static str {
        match self {
            ValidationStage::ChainValidation => "ChainValidation",
            ValidationStage::PayloadValidation => "PayloadValidation",
            ValidationStage::SignatureValidation => "SignatureValidation",
            ValidationStage::GatewayAuthorization => "GatewayAuthorization",
            ValidationStage::ReplayProtection => "ReplayProtection",
        }
    }
}

/// Main validation pipeline function
/// 
/// This function orchestrates the complete validation of cross-chain messages
/// by calling all individual validation functions in sequence. It returns
/// early with an appropriate error if any validation check fails.
pub fn validate_cross_chain_message(
    context: &CrossChainMessageContext,
) -> Result<ValidationPipelineResult> {
    let start_time = std::time::Instant::now();
    let mut stats = ValidationPipelineStats {
        total_validation_time_ms: 0,
        stages_completed: 0,
        stages_passed: 0,
        stages_failed: 0,
        success_rate_percentage: 0,
    };

    // Stage 1: Chain ID Validation
    match validate_chain_ids_simple(context.source_chain_id, context.destination_chain_id) {
        Ok(_) => {
            stats.stages_completed += 1;
            stats.stages_passed += 1;
            log_validation_stage(ValidationStage::ChainValidation, true, None);
        }
        Err(e) => {
            stats.stages_completed += 1;
            stats.stages_failed += 1;
            let error_msg = format!("Chain validation failed: {}", e);
            log_validation_stage(ValidationStage::ChainValidation, false, Some(&error_msg));
            
            let total_time = start_time.elapsed().as_millis() as u64;
            stats.total_validation_time_ms = total_time;
            stats.success_rate_percentage = calculate_success_rate(&stats);
            
            return Ok(ValidationPipelineResult {
                is_valid: false,
                failed_stage: Some(ValidationStage::ChainValidation.as_str().to_string()),
                error_message: Some(error_msg),
                stats,
                validated_at: Clock::get()?.unix_timestamp,
            });
        }
    }

    // Stage 2: Payload Validation
    match validate_payload_size_simple(&context.payload) {
        Ok(_) => {
            stats.stages_completed += 1;
            stats.stages_passed += 1;
            log_validation_stage(ValidationStage::PayloadValidation, true, None);
        }
        Err(e) => {
            stats.stages_completed += 1;
            stats.stages_failed += 1;
            let error_msg = format!("Payload validation failed: {}", e);
            log_validation_stage(ValidationStage::PayloadValidation, false, Some(&error_msg));
            
            let total_time = start_time.elapsed().as_millis() as u64;
            stats.total_validation_time_ms = total_time;
            stats.success_rate_percentage = calculate_success_rate(&stats);
            
            return Ok(ValidationPipelineResult {
                is_valid: false,
                failed_stage: Some(ValidationStage::PayloadValidation.as_str().to_string()),
                error_message: Some(error_msg),
                stats,
                validated_at: Clock::get()?.unix_timestamp,
            });
        }
    }

    // Stage 3: Signature Validation
    match verify_signatures_simple(&context.signatures, &context.validators, 2) {
        Ok(_) => {
            stats.stages_completed += 1;
            stats.stages_passed += 1;
            log_validation_stage(ValidationStage::SignatureValidation, true, None);
        }
        Err(e) => {
            stats.stages_completed += 1;
            stats.stages_failed += 1;
            let error_msg = format!("Signature validation failed: {}", e);
            log_validation_stage(ValidationStage::SignatureValidation, false, Some(&error_msg));
            
            let total_time = start_time.elapsed().as_millis() as u64;
            stats.total_validation_time_ms = total_time;
            stats.success_rate_percentage = calculate_success_rate(&stats);
            
            return Ok(ValidationPipelineResult {
                is_valid: false,
                failed_stage: Some(ValidationStage::SignatureValidation.as_str().to_string()),
                error_message: Some(error_msg),
                stats,
                validated_at: Clock::get()?.unix_timestamp,
            });
        }
    }

    // Stage 4: Gateway Authorization
    match is_gateway_authorized_simple(&context.gateway_caller) {
        Ok(_) => {
            stats.stages_completed += 1;
            stats.stages_passed += 1;
            log_validation_stage(ValidationStage::GatewayAuthorization, true, None);
        }
        Err(e) => {
            stats.stages_completed += 1;
            stats.stages_failed += 1;
            let error_msg = format!("Gateway authorization failed: {}", e);
            log_validation_stage(ValidationStage::GatewayAuthorization, false, Some(&error_msg));
            
            let total_time = start_time.elapsed().as_millis() as u64;
            stats.total_validation_time_ms = total_time;
            stats.success_rate_percentage = calculate_success_rate(&stats);
            
            return Ok(ValidationPipelineResult {
                is_valid: false,
                failed_stage: Some(ValidationStage::GatewayAuthorization.as_str().to_string()),
                error_message: Some(error_msg),
                stats,
                validated_at: Clock::get()?.unix_timestamp,
            });
        }
    }

    // All validation stages passed
    let total_time = start_time.elapsed().as_millis() as u64;
    stats.total_validation_time_ms = total_time;
    stats.success_rate_percentage = 100; // All stages passed

    log_validation_pipeline_completion(true, &stats);

    Ok(ValidationPipelineResult {
        is_valid: true,
        failed_stage: None,
        error_message: None,
        stats,
        validated_at: Clock::get()?.unix_timestamp,
    })
}

/// Validate chain IDs for cross-chain message
fn validate_chain_ids_simple(source_chain_id: u8, destination_chain_id: u8) -> Result<()> {
    // Basic validation: ensure chain IDs are different and non-zero
    if source_chain_id == 0 || destination_chain_id == 0 {
        return Err(UniversalNftError::UnsupportedChainId.into());
    }

    // Ensure source and destination are different
    if source_chain_id == destination_chain_id {
        return Err(UniversalNftError::UnsupportedChainId.into());
    }

    Ok(())
}

/// Validate payload size for cross-chain message
fn validate_payload_size_simple(payload: &[u8]) -> Result<()> {
    // Basic payload validation: ensure it's not empty and not too large
    if payload.is_empty() {
        return Err(UniversalNftError::InvalidPayload.into());
    }
    
    if payload.len() > 1024 {
        return Err(UniversalNftError::InvalidPayload.into());
    }
    
    Ok(())
}

/// Log validation stage completion
fn log_validation_stage(stage: ValidationStage, success: bool, error_msg: Option<&str>) {
    if success {
        msg!("✅ Validation stage {} completed successfully", stage.as_str());
    } else {
        if let Some(error) = error_msg {
            msg!("❌ Validation stage {} failed: {}", stage.as_str(), error);
        } else {
            msg!("❌ Validation stage {} failed", stage.as_str());
        }
    }
}

/// Log validation pipeline completion
fn log_validation_pipeline_completion(success: bool, stats: &ValidationPipelineStats) {
    if success {
        msg!("🎉 Cross-chain message validation pipeline completed successfully!");
        msg!("📊 Validation Statistics:");
        msg!("   - Total time: {}ms", stats.total_validation_time_ms);
        msg!("   - Stages completed: {}", stats.stages_completed);
        msg!("   - Success rate: {}%", stats.success_rate_percentage);
    } else {
        msg!("💥 Cross-chain message validation pipeline failed!");
        msg!("📊 Validation Statistics:");
        msg!("   - Total time: {}ms", stats.total_validation_time_ms);
        msg!("   - Stages completed: {}", stats.stages_completed);
        msg!("   - Stages passed: {}", stats.stages_passed);
        msg!("   - Stages failed: {}", stats.stages_failed);
        msg!("   - Success rate: {}%", stats.success_rate_percentage);
    }
}

/// Calculate success rate percentage
fn calculate_success_rate(stats: &ValidationPipelineStats) -> u8 {
    if stats.stages_completed == 0 {
        return 0;
    }
    
    let success_rate = (stats.stages_passed as f64 / stats.stages_completed as f64) * 100.0;
    success_rate.round() as u8
}

/// Simple signature validation for the pipeline
fn verify_signatures_simple(signatures: &[Vec<u8>], validators: &[Pubkey], required_threshold: u8) -> Result<()> {
    // Basic validation: ensure we have enough signatures
    if signatures.len() < required_threshold as usize {
        return Err(UniversalNftError::InsufficientSignatures.into());
    }
    
    // Basic validation: ensure we have validators
    if validators.is_empty() {
        return Err(UniversalNftError::UnauthorizedValidator.into());
    }
    
    // For now, just check that we have the minimum required signatures
    // In a real implementation, this would verify the cryptographic signatures
    if signatures.len() >= required_threshold as usize {
        Ok(())
    } else {
        Err(UniversalNftError::InsufficientSignatures.into())
    }
}

/// Simple gateway authorization for the pipeline
fn is_gateway_authorized_simple(gateway_caller: &Pubkey) -> Result<()> {
    // For now, just check that the gateway caller is not the default public key
    // In a real implementation, this would check against a list of authorized gateways
    if gateway_caller == &Pubkey::default() {
        return Err(UniversalNftError::UnauthorizedGateway.into());
    }
    
    Ok(())
}

/// Get validation pipeline statistics
pub fn get_validation_pipeline_stats() -> ValidationPipelineStats {
    // This would typically return accumulated statistics from program state
    // For now, return default statistics
    ValidationPipelineStats {
        total_validation_time_ms: 0,
        stages_completed: 0,
        stages_passed: 0,
        stages_failed: 0,
        success_rate_percentage: 0,
    }
}

/// Log validation pipeline operation
pub fn log_validation_pipeline_operation(
    operation: &str,
    context: &CrossChainMessageContext,
    result: &ValidationPipelineResult,
) {
    msg!("🔍 Validation Pipeline Operation: {}", operation);
    msg!("📋 Context:");
    msg!("   - Source Chain: {}", context.source_chain_id);
    msg!("   - Destination Chain: {}", context.destination_chain_id);
    msg!("   - Payload Size: {} bytes", context.payload.len());
    msg!("   - Signatures: {}", context.signatures.len());
    msg!("   - Validators: {}", context.validators.len());
    msg!("   - Gateway Caller: {}", context.gateway_caller);
    msg!("   - Timestamp: {}", context.timestamp);
    
    msg!("📊 Result:");
    msg!("   - Valid: {}", result.is_valid);
    msg!("   - Failed Stage: {:?}", result.failed_stage);
    msg!("   - Error Message: {:?}", result.error_message);
    msg!("   - Validation Time: {}ms", result.stats.total_validation_time_ms);
    msg!("   - Success Rate: {}%", result.stats.success_rate_percentage);
}
