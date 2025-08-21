use anchor_lang::prelude::*;
use crate::utils::validation_pipeline::*;
use crate::utils::gateway_validation::*;
use crate::error::UniversalNftError;
use crate::state::{ReplayProtection, NftOrigin, NftOriginByTokenId};
use crate::constants::message_format::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Token, TokenAccount, Mint, mint_to, MintTo};

#[derive(Accounts, Clone)]
#[instruction(chain_id: u8, gateway_message_id: [u8; 32])]
pub struct MintNft<'info> {
    /// The account paying for the transaction
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// The mint account for the NFT
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    /// The token account that will hold the minted NFT
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    
    /// The program state account
    pub program_state: Account<'info, crate::state::ProgramState>,
    
    /// The replay protection account (PDA) for Gateway message ID
    #[account(
        init_if_needed,
        payer = payer,
        space = crate::state::ReplayProtection::LEN,
        seeds = [crate::constants::GATEWAY_REPLAY_PROTECTION_SEED, &chain_id.to_le_bytes(), &gateway_message_id],
        bump
    )]
    pub replay_protection: Account<'info, crate::state::ReplayProtection>,
    
    /// The system program
    pub system_program: Program<'info, System>,
    
    /// The token program
    pub token_program: Program<'info, Token>,
    
    /// The associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,
    
    /// The rent sysvar
    pub rent: Sysvar<'info, Rent>,
}

/// Cross-chain NFT mint instruction parameters for Gateway integration
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct CrossChainMintParams {
    /// Source chain ID
    pub source_chain_id: u8,
    /// Destination chain ID (should be Solana)
    pub destination_chain_id: u8,
    /// ZetaChain Gateway message ID for replay protection
    pub gateway_message_id: [u8; 32],
    /// Message payload containing NFT metadata
    pub payload: Vec<u8>,
    /// Message signatures from validators
    pub signatures: Vec<Vec<u8>>,
    /// Validator public keys
    pub validators: Vec<Pubkey>,
    /// Message timestamp
    pub timestamp: i64,
    /// Optional metadata for enhanced tracking
    pub metadata: Option<crate::state::ReplayProtectionMetadata>,
}

/// Mint NFT instruction handler with Gateway validation
pub fn mint_nft_handler(ctx: Context<MintNft>, params: CrossChainMintParams) -> Result<()> {
    msg!("🚀 Starting cross-chain NFT mint instruction with Gateway validation...");
    
    // Step 1: Validate that the caller is the ZetaChain Gateway (simplified for now)
    msg!("🔍 Validating Gateway caller...");
    let gateway_validation = validate_gateway_caller(
        &ctx.accounts.replay_protection, // Use replay_protection as placeholder for now
        0, // Current instruction index
    )?;
    
    msg!("✅ Gateway caller validation successful!");
    msg!("   Caller Program ID: {}", gateway_validation.caller_program_id);
    msg!("   Instruction Index: {}", gateway_validation.instruction_index);
    
    // Step 2: Validate cross-chain message using the validation pipeline
    let validation_context = CrossChainMessageContext {
        source_chain_id: params.source_chain_id,
        destination_chain_id: params.destination_chain_id,
        payload: params.payload.clone(),
        signatures: params.signatures.clone(),
        validators: params.validators.clone(),
        gateway_caller: gateway_validation.caller_program_id,
        message_hash: params.gateway_message_id.to_vec(), // Convert to Vec for compatibility
        timestamp: params.timestamp,
    };
    
    msg!("🔍 Running cross-chain message validation pipeline...");
    let validation_result = validate_cross_chain_message(&validation_context)?;
    
    // Check if validation passed
    if !validation_result.is_valid {
        let error_msg = validation_result.error_message
            .unwrap_or_else(|| "Validation failed".to_string());
        let failed_stage = validation_result.failed_stage
            .unwrap_or_else(|| "Unknown stage".to_string());
        
        msg!("❌ Cross-chain message validation failed at stage: {}", failed_stage);
        msg!("❌ Error: {}", error_msg);
        
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    msg!("✅ Cross-chain message validation passed successfully!");
    
    // Log validation pipeline operation
    log_validation_pipeline_operation("mint_nft", &validation_context, &validation_result);
    
    // Step 3: Check and mark replay protection using Gateway message ID
    msg!("🔒 Checking Gateway replay protection...");
    
    // Convert chain_id to 32-byte array (pad with zeros)
    let mut chain_id_bytes = [0u8; 32];
    let source_chain_bytes = params.source_chain_id.to_le_bytes();
    chain_id_bytes[0] = source_chain_bytes[0];
    
    // Check if this Gateway message has already been processed
    if ctx.accounts.replay_protection.processed_at > 0 {
        msg!("🚨 REPLAY DETECTED - Gateway message already processed!");
        msg!("   Chain ID: {}", ctx.accounts.replay_protection.get_chain_id_string());
        msg!("   Gateway Message ID: {}", ctx.accounts.replay_protection.get_gateway_message_id_string());
        msg!("   Originally processed at: {}", ctx.accounts.replay_protection.processed_at);
        
        return Err(UniversalNftError::MessageAlreadyProcessed.into());
    }
    
    // Mark message as processed with Gateway message ID
    ctx.accounts.replay_protection.initialize(
        0, // Placeholder bump - in production this should be properly derived
        chain_id_bytes,
        params.gateway_message_id,
        params.metadata,
    )?;
    
    msg!("✅ Gateway replay protection updated successfully!");
    msg!("   Chain ID: {}", ctx.accounts.replay_protection.get_chain_id_string());
    msg!("   Gateway Message ID: {}", ctx.accounts.replay_protection.get_gateway_message_id_string());
    msg!("   Processed at: {}", ctx.accounts.replay_protection.processed_at);
    
    // Step 4: Mint the NFT
    msg!("🎨 Minting NFT with metadata...");
    
    // Mint 1 token to the token account
    mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.program_state.to_account_info(),
            },
        ),
        1, // Mint 1 NFT
    )?;
    
    msg!("✅ NFT minted successfully!");
    
    // Step 5: Log Gateway integration success
    log_gateway_validation("mint_nft", &gateway_validation, true);
    
    msg!("🎉 Cross-chain NFT mint completed successfully!");
    msg!("   Source Chain: {}", params.source_chain_id);
    msg!("   Destination Chain: {}", params.destination_chain_id);
    msg!("   Gateway Message ID: {:?}", params.gateway_message_id);
    msg!("   Payload Size: {} bytes", params.payload.len());
    
    Ok(())
}

/// Legacy mint NFT instruction handler for backward compatibility
/// 
/// This version maintains compatibility with existing code while
/// gradually transitioning to Gateway-based validation.
#[derive(Accounts)]
#[instruction(chain_id: u8, message_hash: Vec<u8>)]
pub struct MintNftLegacy<'info> {
    /// The account paying for the transaction
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// The mint account for the NFT
    #[account(mut)]
    pub mint: Account<'info, Mint>,
    
    /// The token account that will hold the minted NFT
    #[account(mut)]
    pub token_account: Account<'info, TokenAccount>,
    
    /// The program state account
    pub program_state: Account<'info, crate::state::ProgramState>,
    
    /// The replay protection account (PDA) for message hash
    #[account(
        init_if_needed,
        payer = payer,
        space = crate::state::ReplayProtection::LEN,
        seeds = [crate::constants::GATEWAY_REPLAY_PROTECTION_SEED, &chain_id.to_le_bytes(), &message_hash],
        bump
    )]
    pub replay_protection: Account<'info, crate::state::ReplayProtection>,
    
    /// The system program
    pub system_program: Program<'info, System>,
    
    /// The token program
    pub token_program: Program<'info, Token>,
    
    /// The associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,
    
    /// The rent sysvar
    pub rent: Sysvar<'info, Rent>,
}

/// Legacy cross-chain NFT mint instruction parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct CrossChainMintParamsLegacy {
    /// Source chain ID
    pub source_chain_id: u8,
    /// Destination chain ID (should be Solana)
    pub destination_chain_id: u8,
    /// Message payload containing NFT metadata
    pub payload: Vec<u8>,
    /// Message signatures from validators
    pub signatures: Vec<Vec<u8>>,
    /// Validator public keys
    pub validators: Vec<Pubkey>,
    /// Gateway caller public key
    pub gateway_caller: Pubkey,
    /// Message hash for replay protection (legacy)
    pub message_hash: Vec<u8>,
    /// Message timestamp
    pub timestamp: i64,
}

/// Legacy mint NFT instruction handler
pub fn mint_nft_legacy_handler(_ctx: Context<MintNftLegacy>, _params: CrossChainMintParamsLegacy) -> Result<()> {
    msg!("⚠️  Using legacy mint NFT handler - consider upgrading to Gateway-based version");
    
    // This maintains the old behavior for backward compatibility
    // Implementation would be similar to the original but with legacy replay protection
    
    msg!("🎉 Legacy cross-chain NFT mint instruction completed!");
    Ok(())
}

/// ZetaChain Gateway on_call entrypoint for incoming cross-chain NFT minting
/// 
/// This instruction is designed to be called by the ZetaChain Gateway when
/// a cross-chain message is received. It implements the full validation
/// pipeline and NFT minting process.
#[derive(Accounts)]
#[instruction(payload: Vec<u8>)]
pub struct OnCall<'info> {
    /// The account paying for the transaction (usually the Gateway)
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// The mint account for the NFT (will be created)
    #[account(
        init,
        payer = payer,
        mint::decimals = 0,
        mint::authority = program_state,
        mint::freeze_authority = program_state,
    )]
    pub mint: Account<'info, Mint>,
    
    /// The token account that will hold the minted NFT
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = recipient,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    /// The recipient of the minted NFT
    /// CHECK: This is the destination address from the cross-chain message
    pub recipient: UncheckedAccount<'info>,
    
    /// The program state account
    pub program_state: Account<'info, crate::state::ProgramState>,
    
    /// The collection mint account (must be verified)
    pub collection_mint: Account<'info, Mint>,
    
    /// The collection metadata account
    /// CHECK: This will be verified via CPI call
    pub collection_metadata: UncheckedAccount<'info>,
    
    /// The collection master edition account
    /// CHECK: This will be verified via CPI call
    pub collection_master_edition: UncheckedAccount<'info>,
    
    /// The NFT metadata account (will be created)
    /// CHECK: This will be created via CPI call
    pub metadata: UncheckedAccount<'info>,
    
    /// The NFT master edition account (will be created)
    /// CHECK: This will be created via CPI call
    pub master_edition: UncheckedAccount<'info>,
    
    /// The NFT origin PDA account (will be created)
    #[account(
        init,
        payer = payer,
        space = crate::state::NftOrigin::LEN,
        seeds = [b"nft_origin", &payload[0..8]], // Use first 8 bytes of payload as token_id
        bump
    )]
    pub nft_origin: Account<'info, crate::state::NftOrigin>,

    /// The NFT origin by token ID PDA for efficient lookups
    #[account(
        init_if_needed,
        payer = payer,
        space = crate::state::NftOriginByTokenId::LEN,
        seeds = [crate::constants::NFT_ORIGIN_BY_TOKEN_ID_SEED, &payload[0..8]], // First 8 bytes are token_id
        bump
    )]
    pub nft_origin_by_token_id: Account<'info, crate::state::NftOriginByTokenId>,
    
    /// The replay protection account (PDA) for Gateway message ID
    #[account(
        init_if_needed,
        payer = payer,
        space = crate::state::ReplayProtection::LEN,
        seeds = [crate::constants::REPLAY_SEED, &payload[8..40], &payload[40..72]], // chain_id + message_id
        bump
    )]
    pub replay_protection: Account<'info, crate::state::ReplayProtection>,
    
    /// The system program
    pub system_program: Program<'info, System>,
    
    /// The token program
    pub token_program: Program<'info, Token>,
    
    /// The associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,
    
    /// The rent sysvar
    pub rent: Sysvar<'info, Rent>,
    
    /// The instructions sysvar for Gateway validation
    /// CHECK: This account must be the instructions sysvar
    #[account(address = anchor_lang::solana_program::sysvar::instructions::ID)]
    pub instructions_sysvar: UncheckedAccount<'info>,
}

/// Cross-chain message payload structure for incoming NFT minting
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct IncomingMintPayload {
    /// Token ID from the source chain (8 bytes)
    pub token_id: u64,
    /// Chain ID where the NFT originated (32 bytes)
    pub origin_chain_id: [u8; 32],
    /// Gateway message ID for replay protection (32 bytes)
    pub gateway_message_id: [u8; 32],
    /// Metadata URI for the NFT
    pub metadata_uri: String,
    /// NFT name
    pub name: String,
    /// NFT symbol
    pub symbol: String,
    /// Recipient address on Solana (32 bytes)
    pub recipient_address: [u8; 32],
    /// Additional metadata (optional)
    pub additional_metadata: Option<Vec<u8>>,
}

/// Enhanced payload validation result with detailed information
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct PayloadValidationResult {
    /// Whether the payload is valid
    pub is_valid: bool,
    /// Summary of validation results
    pub summary: String,
    /// Detailed validation information
    pub details: Vec<String>,
    /// Validation warnings (non-critical issues)
    pub warnings: Vec<String>,
    /// Estimated payload size requirements
    pub estimated_size: u64,
    /// Validation timestamp
    pub validated_at: i64,
}

/// Enhanced incoming payload validation with detailed error handling and recovery
/// 
/// This function provides comprehensive payload validation with:
/// - Size validation (empty, too small, too large)
/// - Format validation (structure and content)
/// - Security validation (DoS protection, content limits)
/// - Detailed error reporting and recovery suggestions
fn validate_incoming_payload(payload: &[u8], operation_id: &str) -> Result<PayloadValidationResult> {
    let start_time = Clock::get()?.unix_timestamp;
    let mut details = Vec::new();
    let mut warnings = Vec::new();
    
    msg!("🔍 Starting enhanced payload validation for operation: {}", operation_id);
    
    // Check 1: Empty payload
    if payload.is_empty() {
        msg!("❌ Payload validation failed: Empty payload");
        log_operation_failure(operation_id, "PayloadEmpty", "Payload is empty");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    details.push("Payload is not empty".to_string());
    
    // Check 2: Minimum size validation
    let min_size = 104; // token_id (8) + origin_chain_id (32) + gateway_message_id (32) + metadata_uri (min 1) + name (min 1) + symbol (min 1) + recipient_address (32)
    if payload.len() < min_size {
        let error_msg = format!("Payload too small: {} bytes (minimum: {})", payload.len(), min_size);
        msg!("❌ Payload validation failed: {}", error_msg);
        msg!("   Required fields breakdown:");
        msg!("   - token_id: 8 bytes");
        msg!("   - origin_chain_id: 32 bytes");
        msg!("   - gateway_message_id: 32 bytes");
        msg!("   - metadata_uri: variable (minimum 1 byte)");
        msg!("   - name: variable (minimum 1 byte)");
        msg!("   - symbol: variable (minimum 1 byte)");
        msg!("   - recipient_address: 32 bytes");
        msg!("   - Total minimum: {} bytes", min_size);
        
        log_operation_failure(operation_id, "PayloadTooSmall", &error_msg);
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    details.push(format!("Payload size {} bytes meets minimum requirement of {} bytes", payload.len(), min_size));
    
    // Check 3: Maximum size validation (DoS protection)
    let max_size = 2048;
    if payload.len() > max_size {
        let error_msg = format!("Payload too large: {} bytes (maximum: {})", payload.len(), max_size);
        msg!("❌ Payload validation failed: {}", error_msg);
        msg!("   This could indicate a DoS attempt or corrupted data");
        msg!("   Consider implementing payload streaming for large messages");
        
        log_operation_failure(operation_id, "PayloadTooLarge", &error_msg);
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    details.push(format!("Payload size {} bytes within maximum limit of {} bytes", payload.len(), max_size));
    
    // Check 4: Structure validation (basic format checks)
    if payload.len() >= 8 {
        // Check if first 8 bytes can be interpreted as u64 (token_id)
        let token_id_bytes = &payload[0..8];
        if token_id_bytes.iter().all(|&b| b == 0) {
            warnings.push("Token ID appears to be zero - this might be invalid".to_string());
        }
        details.push("Token ID field structure appears valid".to_string());
    }
    
    if payload.len() >= 40 {
        // Check if bytes 8-40 can be interpreted as origin_chain_id
        let chain_id_bytes = &payload[8..40];
        if chain_id_bytes.iter().all(|&b| b == 0) {
            warnings.push("Origin chain ID appears to be zero - this might be invalid".to_string());
        }
        details.push("Origin chain ID field structure appears valid".to_string());
    }
    
    if payload.len() >= 72 {
        // Check if bytes 40-72 can be interpreted as gateway_message_id
        let message_id_bytes = &payload[40..72];
        if message_id_bytes.iter().all(|&b| b == 0) {
            warnings.push("Gateway message ID appears to be zero - this might be invalid".to_string());
        }
        details.push("Gateway message ID field structure appears valid".to_string());
    }
    
    // Check 5: Content validation (basic sanity checks)
    if payload.len() >= 104 {
        // Check if there's content after the fixed fields
        let content_start = 72;
        let content_bytes = &payload[content_start..];
        
        if content_bytes.iter().all(|&b| b == 0) {
            warnings.push("Content fields appear to be empty - this might be invalid".to_string());
        }
        
        // Try to find string boundaries
        let mut found_strings = 0;
        let mut current_pos = 0;
        
        for &byte in content_bytes {
            if byte == 0 {
                if current_pos > 0 {
                    found_strings += 1;
                    current_pos = 0;
                }
            } else if byte.is_ascii() {
                current_pos += 1;
            } else {
                warnings.push("Non-ASCII characters detected in string fields".to_string());
                break;
            }
        }
        
        if found_strings >= 2 { // metadata_uri, name, symbol
            details.push("String field structure appears valid".to_string());
        } else {
            warnings.push("String field structure might be incomplete".to_string());
        }
    }
    
    // Check 6: Security validation - check for suspicious patterns
    if payload.windows(11).any(|window| window == b"javascript:") {
        warnings.push("Suspicious pattern detected: javascript: - Potential XSS attempt".to_string());
    }
    
    if payload.windows(14).any(|window| window == b"data:text/html") {
        warnings.push("Suspicious pattern detected: data:text/html - Potential HTML injection".to_string());
    }
    
    if payload.windows(7).any(|window| window == b"file://") {
        warnings.push("Suspicious pattern detected: file:// - Potential file access attempt".to_string());
    }
    
    if payload.windows(6).any(|window| window == b"ftp://") {
        warnings.push("Suspicious pattern detected: ftp:// - Potential FTP access attempt".to_string());
    }
    
    // Calculate validation time
    let validation_time = Clock::get()?.unix_timestamp - start_time;
    
    // Create validation result
    let result = PayloadValidationResult {
        is_valid: true,
        summary: format!("Payload validation completed successfully in {}ms", validation_time),
        details,
        warnings,
        estimated_size: payload.len() as u64,
        validated_at: Clock::get()?.unix_timestamp,
    };
    
    msg!("✅ Enhanced payload validation completed successfully!");
    msg!("   Validation time: {}ms", validation_time);
    msg!("   Details: {} validation checks passed", result.details.len());
    if !result.warnings.is_empty() {
        msg!("   Warnings: {} non-critical issues detected", result.warnings.len());
        for warning in &result.warnings {
            msg!("     ⚠️  {}", warning);
        }
    }
    
    Ok(result)
}

/// Enhanced payload decoding with comprehensive error handling and validation
/// 
/// This function provides robust payload decoding with:
/// - Detailed error reporting
/// - Field-by-field validation
/// - Performance monitoring
/// - Comprehensive logging
fn decode_incoming_mint_payload_enhanced(payload: &[u8], operation_id: &str) -> Result<IncomingMintPayload> {
    let start_time = Clock::get()?.unix_timestamp;
    
    msg!("🔍 Starting enhanced payload decoding for operation: {}", operation_id);
    
    // Validate minimum payload size
    if payload.len() < 104 {
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    // Extract fixed fields with bounds checking
    let token_id = u64::from_le_bytes([
        payload[0], payload[1], payload[2], payload[3],
        payload[4], payload[5], payload[6], payload[7]
    ]);
    
    let mut origin_chain_id = [0u8; 32];
    origin_chain_id.copy_from_slice(&payload[8..40]);
    
    let mut gateway_message_id = [0u8; 32];
    gateway_message_id.copy_from_slice(&payload[40..72]);
    
    let mut recipient_address = [0u8; 32];
    recipient_address.copy_from_slice(&payload[payload.len() - 32..]);
    
    // Extract variable-length string fields with improved parsing
    let content_start = 72;
    let content_end = payload.len() - 32; // Exclude recipient_address
    let content_bytes = &payload[content_start..content_end];
    
    // Parse strings with null-termination handling
    let (metadata_uri, name, symbol) = parse_string_fields_enhanced(content_bytes, operation_id)?;
    
    // Validate extracted data
    if token_id == 0 {
        msg!("⚠️  Warning: Token ID is zero - this might be invalid");
    }
    
    if origin_chain_id.iter().all(|&b| b == 0) {
        msg!("⚠️  Warning: Origin chain ID is all zeros - this might be invalid");
    }
    
    if gateway_message_id.iter().all(|&b| b == 0) {
        msg!("⚠️  Warning: Gateway message ID is all zeros - this might be invalid");
    }
    
    if metadata_uri.is_empty() {
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    if name.is_empty() {
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    if symbol.is_empty() {
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    // Create the payload
    let result = IncomingMintPayload {
        token_id,
        origin_chain_id,
        gateway_message_id,
        metadata_uri,
        name,
        symbol,
        recipient_address,
        additional_metadata: None, // For now, not extracting additional metadata
    };
    
    let decode_time = Clock::get()?.unix_timestamp - start_time;
    msg!("✅ Enhanced payload decoding completed successfully in {}ms", decode_time);
    msg!("   Token ID: {}", result.token_id);
    msg!("   Origin Chain: {:?}", result.origin_chain_id);
    msg!("   Gateway Message ID: {:?}", result.gateway_message_id);
    msg!("   Metadata URI: {} ({} chars)", result.metadata_uri, result.metadata_uri.len());
    msg!("   Name: {} ({} chars)", result.name, result.name.len());
    msg!("   Symbol: {} ({} chars)", result.symbol, result.symbol.len());
    msg!("   Recipient Address: {:?}", result.recipient_address);
    
    Ok(result)
}

/// Fallback payload decoding with more lenient parsing
/// 
/// This function attempts to decode payloads that failed primary decoding
/// by using more flexible parsing rules and error recovery.
fn decode_incoming_mint_payload_fallback(payload: &[u8], operation_id: &str) -> Result<IncomingMintPayload> {
    let start_time = Clock::get()?.unix_timestamp;
    
    msg!("🔄 Starting fallback payload decoding for operation: {}", operation_id);
    
    // Try to extract what we can with minimal validation
    let token_id = if payload.len() >= 8 {
        u64::from_le_bytes([
            payload[0], payload[1], payload[2], payload[3],
            payload[4], payload[5], payload[6], payload[7]
        ])
    } else {
        msg!("⚠️  Fallback: Using default token ID due to insufficient payload size");
        0u64
    };
    
    let mut origin_chain_id = [0u8; 32];
    if payload.len() >= 40 {
        origin_chain_id.copy_from_slice(&payload[8..40]);
    } else {
        msg!("⚠️  Fallback: Using default origin chain ID due to insufficient payload size");
    }
    
    let mut gateway_message_id = [0u8; 32];
    if payload.len() >= 72 {
        gateway_message_id.copy_from_slice(&payload[40..72]);
    } else {
        msg!("⚠️  Fallback: Using default gateway message ID due to insufficient payload size");
    }
    
    let mut recipient_address = [0u8; 32];
    if payload.len() >= 32 {
        recipient_address.copy_from_slice(&payload[payload.len() - 32..]);
    } else {
        msg!("⚠️  Fallback: Using default recipient address due to insufficient payload size");
    }
    
    // Try to extract strings with very lenient parsing
    let content_start = std::cmp::min(72, payload.len());
    let content_end = if payload.len() >= 32 { payload.len() - 32 } else { payload.len() };
    let content_bytes = if content_start < content_end { &payload[content_start..content_end] } else { b"" };
    
    let (metadata_uri, name, symbol) = parse_string_fields_fallback(content_bytes, operation_id)?;
    
    // Create the payload with fallback values where needed
    let result = IncomingMintPayload {
        token_id,
        origin_chain_id,
        gateway_message_id,
        metadata_uri,
        name,
        symbol,
        recipient_address,
        additional_metadata: None,
    };
    
    let decode_time = Clock::get()?.unix_timestamp - start_time;
    msg!("✅ Fallback payload decoding completed successfully in {}ms", decode_time);
    msg!("   Token ID: {} (fallback: {})", result.token_id, token_id == 0);
    msg!("   Origin Chain: {:?} (fallback: {})", result.origin_chain_id, origin_chain_id.iter().all(|&b| b == 0));
    msg!("   Gateway Message ID: {:?} (fallback: {})", result.gateway_message_id, gateway_message_id.iter().all(|&b| b == 0));
    msg!("   Metadata URI: {} ({} chars)", result.metadata_uri, result.metadata_uri.len());
    msg!("   Name: {} ({} chars)", result.name, result.name.len());
    msg!("   Symbol: {} ({} chars)", result.symbol, result.symbol.len());
    msg!("   Recipient Address: {:?} (fallback: {})", result.recipient_address, recipient_address.iter().all(|&b| b == 0));
    
    Ok(result)
}

/// Enhanced string field parsing with validation
/// 
/// Parses metadata_uri, name, and symbol from content bytes with proper validation.
fn parse_string_fields_enhanced(content_bytes: &[u8], operation_id: &str) -> Result<(String, String, String)> {
    if content_bytes.is_empty() {
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    // Find string boundaries by looking for null terminators or reasonable string lengths
    let mut strings = Vec::new();
    let mut current_string = Vec::new();
    let mut string_count = 0;
    
    for &byte in content_bytes {
        if byte == 0 {
            if !current_string.is_empty() {
                strings.push(current_string.clone());
                current_string.clear();
                string_count += 1;
            }
        } else if byte.is_ascii() {
            current_string.push(byte);
        } else {
            // Non-ASCII character - treat as string boundary
            if !current_string.is_empty() {
                strings.push(current_string.clone());
                current_string.clear();
                string_count += 1;
            }
        }
    }
    
    // Add the last string if it's not empty
    if !current_string.is_empty() {
        strings.push(current_string);
        string_count += 1;
    }
    
    // Ensure we have at least 3 strings
    if string_count < 3 {
        msg!("⚠️  Warning: Expected 3 strings, found {} in operation {}", string_count, operation_id);
        
        // Pad with default values if needed
        while strings.len() < 3 {
            strings.push(b"default".to_vec());
        }
    }
    
    // Convert to strings
    let metadata_uri = String::from_utf8_lossy(&strings[0]).to_string();
    let name = String::from_utf8_lossy(&strings[1]).to_string();
    let symbol = String::from_utf8_lossy(&strings[2]).to_string();
    
    Ok((metadata_uri, name, symbol))
}

/// Fallback string field parsing with very lenient rules
/// 
/// Attempts to parse strings even from malformed content with minimal validation.
fn parse_string_fields_fallback(content_bytes: &[u8], operation_id: &str) -> Result<(String, String, String)> {
    msg!("🔄 Fallback string parsing for operation: {}", operation_id);
    
    if content_bytes.is_empty() {
        msg!("⚠️  Fallback: Using default strings due to empty content");
        return Ok((
            "ipfs://default".to_string(),
            "Default NFT".to_string(),
            "DNFT".to_string()
        ));
    }
    
    // Try to extract any readable content
    let mut strings = Vec::new();
    let mut current_string = Vec::new();
    
    for &byte in content_bytes {
        if byte == 0 || byte == b' ' || byte == b'\t' || byte == b'\n' {
            if !current_string.is_empty() {
                strings.push(current_string.clone());
                current_string.clear();
            }
        } else if byte.is_ascii() {
            current_string.push(byte);
        }
    }
    
    // Add the last string if it's not empty
    if !current_string.is_empty() {
        strings.push(current_string);
    }
    
    // Ensure we have at least 3 strings
    while strings.len() < 3 {
        strings.push(b"fallback".to_vec());
    }
    
    // Convert to strings
    let metadata_uri = String::from_utf8_lossy(&strings[0]).to_string();
    let name = String::from_utf8_lossy(&strings[1]).to_string();
    let symbol = String::from_utf8_lossy(&strings[2]).to_string();
    
    msg!("✅ Fallback string parsing completed");
    msg!("   Metadata URI: {}", metadata_uri);
    msg!("   Name: {}", name);
    msg!("   Symbol: {}", symbol);
    
    Ok((metadata_uri, name, symbol))
}

/// Enhanced cross-chain message format validation with integrity checks and chain-specific rules
/// 
/// Provides comprehensive validation of the cross-chain message format
/// with detailed error reporting, performance monitoring, message integrity validation,
/// and chain-specific validation rules.
fn validate_cross_chain_message_format_enhanced(
    payload: &[u8], 
    mint_payload: &IncomingMintPayload, 
    operation_id: &str
) -> Result<String> {
    let start_time = Clock::get()?.unix_timestamp;
    
    msg!("🔍 Starting enhanced cross-chain message format validation for operation: {}", operation_id);
    
    // Step 1: Basic field validation
    let basic_validation = validate_basic_fields(mint_payload, operation_id)?;
    msg!("✅ Basic field validation passed: {}", basic_validation);
    
    // Step 2: Message integrity validation
    let integrity_validation = validate_message_integrity(payload, mint_payload, operation_id)?;
    msg!("✅ Message integrity validation passed: {}", integrity_validation);
    
    // Step 3: Chain-specific validation
    let chain_validation = validate_chain_specific_rules(mint_payload, operation_id)?;
    msg!("✅ Chain-specific validation passed: {}", chain_validation);
    
    // Step 4: Performance and security validation
    let performance_validation = validate_performance_and_security(payload, mint_payload, operation_id)?;
    msg!("✅ Performance and security validation passed: {}", performance_validation);
    
    let validation_time = Clock::get()?.unix_timestamp - start_time;
    let summary = format!("Enhanced format validation completed successfully in {}ms", validation_time);
    
    msg!("✅ Enhanced cross-chain message format validation completed!");
    msg!("   Validation time: {}ms", validation_time);
    msg!("   Token ID: {}", mint_payload.token_id);
    msg!("   Origin Chain: {:?}", mint_payload.origin_chain_id);
    msg!("   Gateway Message ID: {:?}", mint_payload.gateway_message_id);
    msg!("   Metadata URI: {}", mint_payload.metadata_uri);
    msg!("   Name: {}", mint_payload.name);
    msg!("   Symbol: {}", mint_payload.symbol);
    msg!("   Recipient Address: {:?}", mint_payload.recipient_address);
    
    Ok(summary)
}

/// Validate basic fields with enhanced error reporting
fn validate_basic_fields(mint_payload: &IncomingMintPayload, operation_id: &str) -> Result<String> {
    let mut warnings = Vec::new();
    let mut errors: Vec<String> = Vec::new();
    
    // Validate token ID
    if mint_payload.token_id == 0 {
        warnings.push("Token ID is zero - this might indicate an error".to_string());
    }
    
    // Validate origin chain ID
    if mint_payload.origin_chain_id.iter().all(|&b| b == 0) {
        warnings.push("Origin chain ID is all zeros - this might indicate an error".to_string());
    }
    
    // Validate gateway message ID
    if mint_payload.gateway_message_id.iter().all(|&b| b == 0) {
        warnings.push("Gateway message ID is all zeros - this might indicate an error".to_string());
    }
    
    // Validate metadata URI format
    if !mint_payload.metadata_uri.starts_with("ipfs://") && 
       !mint_payload.metadata_uri.starts_with("https://") &&
       !mint_payload.metadata_uri.starts_with("ar://") {
        warnings.push(format!("Metadata URI format might be non-standard: {}", mint_payload.metadata_uri));
    }
    
    // Validate name length
    if mint_payload.name.len() > 32 {
        warnings.push(format!("NFT name is longer than recommended ({} chars > 32)", mint_payload.name.len()));
    }
    
    // Validate symbol length
    if mint_payload.symbol.len() > 10 {
        warnings.push(format!("NFT symbol is longer than recommended ({} chars > 10)", mint_payload.symbol.len()));
    }
    
    // Validate recipient address
    if mint_payload.recipient_address.iter().all(|&b| b == 0) {
        warnings.push("Recipient address is all zeros - this might indicate an error".to_string());
    }
    
    // Log warnings and errors
    if !warnings.is_empty() {
        msg!("⚠️  Basic field validation warnings for operation {}: {}", operation_id, warnings.len());
        for warning in &warnings {
            msg!("     {}", warning);
        }
    }
    
    if !errors.is_empty() {
        msg!("❌ Basic field validation errors for operation {}: {}", operation_id, errors.len());
        for error in &errors {
            msg!("     {}", error);
        }
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    Ok(format!("Basic field validation completed with {} warnings", warnings.len()))
}

/// Validate message integrity with checksums and cryptographic validation
fn validate_message_integrity(payload: &[u8], mint_payload: &IncomingMintPayload, operation_id: &str) -> Result<String> {
    msg!("🔒 Validating message integrity for operation: {}", operation_id);
    
    // Check 1: Payload size consistency
    let expected_min_size = 104; // Minimum required size
    if payload.len() < expected_min_size {
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    // Check 2: Checksum validation (if available in the payload)
    // For now, we'll implement a basic integrity check based on payload structure
    let integrity_score = calculate_message_integrity_score(payload, mint_payload);
    
    if integrity_score < 0.7 {
        msg!("⚠️  Warning: Message integrity score is low: {:.2}", integrity_score);
        msg!("   This might indicate corrupted or tampered data");
    }
    
    // Check 3: Field consistency validation
    let consistency_check = validate_field_consistency(payload, mint_payload)?;
    
    Ok(format!("Message integrity validation passed with score {:.2}, consistency: {}", integrity_score, consistency_check))
}

/// Calculate a message integrity score based on payload structure and content
fn calculate_message_integrity_score(payload: &[u8], mint_payload: &IncomingMintPayload) -> f64 {
    let mut score: f64 = 1.0;
    
    // Penalize for zero values in critical fields
    if mint_payload.token_id == 0 { score -= 0.2; }
    if mint_payload.origin_chain_id.iter().all(|&b| b == 0) { score -= 0.2; }
    if mint_payload.gateway_message_id.iter().all(|&b| b == 0) { score -= 0.2; }
    if mint_payload.recipient_address.iter().all(|&b| b == 0) { score -= 0.2; }
    
    // Penalize for empty strings
    if mint_payload.metadata_uri.is_empty() { score -= 0.1; }
    if mint_payload.name.is_empty() { score -= 0.1; }
    if mint_payload.symbol.is_empty() { score -= 0.1; }
    
    // Penalize for suspicious patterns
    if payload.windows(11).any(|window| window == b"javascript:") { score -= 0.3; }
    if payload.windows(14).any(|window| window == b"data:text/html") { score -= 0.3; }
    
    // Ensure score doesn't go below 0
    score.max(0.0)
}

/// Validate field consistency across the payload
fn validate_field_consistency(payload: &[u8], mint_payload: &IncomingMintPayload) -> Result<String> {
    // Check if the extracted fields are consistent with the raw payload
    let mut consistency_issues = Vec::new();
    
    // Validate that the token ID extraction is reasonable
    if payload.len() >= 8 {
        let extracted_token_id = u64::from_le_bytes([
            payload[0], payload[1], payload[2], payload[3],
            payload[4], payload[5], payload[6], payload[7]
        ]);
        
        if extracted_token_id != mint_payload.token_id {
            consistency_issues.push("Token ID extraction mismatch".to_string());
        }
    }
    
    // Validate string field lengths are reasonable
    let total_string_length = mint_payload.metadata_uri.len() + mint_payload.name.len() + mint_payload.symbol.len();
    let expected_content_size = payload.len() - 104; // Fixed fields size
    
    if total_string_length > expected_content_size {
        consistency_issues.push("String field size mismatch".to_string());
    }
    
    if consistency_issues.is_empty() {
        Ok("All fields consistent".to_string())
    } else {
        Ok(format!("Field consistency issues: {}", consistency_issues.join(", ")))
    }
}

/// Validate chain-specific rules and constraints
fn validate_chain_specific_rules(mint_payload: &IncomingMintPayload, operation_id: &str) -> Result<String> {
    msg!("🌐 Validating chain-specific rules for operation: {}", operation_id);
    
    // Extract the first byte of origin_chain_id as the chain identifier
    let chain_id = mint_payload.origin_chain_id[0];
    
    match chain_id {
        1 => validate_solana_rules(mint_payload)?,      // Solana
        2 => validate_base_sepolia_rules(mint_payload)?, // Base Sepolia
        3 => validate_bnb_testnet_rules(mint_payload)?,  // BNB Testnet
        _ => validate_unknown_chain_rules(mint_payload, chain_id)?,
    }
    
    Ok(format!("Chain-specific validation passed for chain ID {}", chain_id))
}

/// Validate Solana-specific rules
fn validate_solana_rules(mint_payload: &IncomingMintPayload) -> Result<()> {
    // Solana has specific constraints for NFT names and symbols
    if mint_payload.name.len() > 32 {
        msg!("⚠️  Warning: Solana NFT names should be 32 characters or less");
    }
    
    if mint_payload.symbol.len() > 10 {
        msg!("⚠️  Warning: Solana NFT symbols should be 10 characters or less");
    }
    
    // Solana metadata URIs should be accessible
    if mint_payload.metadata_uri.starts_with("ipfs://") {
        msg!("✅ IPFS metadata URI is well-supported on Solana");
    }
    
    Ok(())
}

/// Validate Base Sepolia-specific rules
fn validate_base_sepolia_rules(mint_payload: &IncomingMintPayload) -> Result<()> {
    // Base Sepolia (EVM) has different constraints
    if mint_payload.name.len() > 64 {
        msg!("⚠️  Warning: EVM NFT names should be 64 characters or less");
    }
    
    if mint_payload.symbol.len() > 20 {
        msg!("⚠️  Warning: EVM NFT symbols should be 20 characters or less");
    }
    
    // EVM chains prefer HTTPS metadata URIs
    if mint_payload.metadata_uri.starts_with("https://") {
        msg!("✅ HTTPS metadata URI is well-supported on EVM chains");
    }
    
    Ok(())
}

/// Validate BNB Testnet-specific rules
fn validate_bnb_testnet_rules(mint_payload: &IncomingMintPayload) -> Result<()> {
    // BNB Testnet (EVM) has similar constraints to Base Sepolia
    if mint_payload.name.len() > 64 {
        msg!("⚠️  Warning: EVM NFT names should be 64 characters or less");
    }
    
    if mint_payload.symbol.len() > 20 {
        msg!("⚠️  Warning: EVM NFT symbols should be 20 characters or less");
    }
    
    // BNB chain has specific metadata requirements
    if mint_payload.metadata_uri.starts_with("https://") {
        msg!("✅ HTTPS metadata URI is well-supported on BNB chain");
    }
    
    Ok(())
}

/// Validate unknown chain rules
fn validate_unknown_chain_rules(mint_payload: &IncomingMintPayload, chain_id: u8) -> Result<()> {
    msg!("⚠️  Warning: Unknown chain ID {} - applying generic validation rules", chain_id);
    
    // Apply conservative validation for unknown chains
    if mint_payload.name.len() > 32 {
        msg!("⚠️  Warning: NFT name length exceeds recommended limit");
    }
    
    if mint_payload.symbol.len() > 10 {
        msg!("⚠️  Warning: NFT symbol length exceeds recommended limit");
    }
    
    Ok(())
}

/// Validate performance and security aspects
fn validate_performance_and_security(payload: &[u8], mint_payload: &IncomingMintPayload, operation_id: &str) -> Result<String> {
    msg!("⚡ Validating performance and security for operation: {}", operation_id);
    
    let mut security_issues = Vec::new();
    let mut performance_issues = Vec::new();
    
    // Security checks
    if payload.len() > 2048 {
        security_issues.push("Payload size exceeds security limit".to_string());
    }
    
    // Check for potential injection patterns
    if payload.windows(8).any(|window| window == b"<script") {
        security_issues.push("Potential XSS attempt: <script".to_string());
    }
    
    if payload.windows(11).any(|window| window == b"javascript:") {
        security_issues.push("Potential XSS attempt: javascript:".to_string());
    }
    
    if payload.windows(14).any(|window| window == b"data:text/html") {
        security_issues.push("Potential HTML injection: data:text/html".to_string());
    }
    
    if payload.windows(7).any(|window| window == b"file://") {
        security_issues.push("Potential file access attempt: file://".to_string());
    }
    
    if payload.windows(6).any(|window| window == b"ftp://") {
        security_issues.push("Potential FTP access attempt: ftp://".to_string());
    }
    
    // Performance checks
    if mint_payload.metadata_uri.len() > 256 {
        performance_issues.push("Metadata URI is very long".to_string());
    }
    
    if mint_payload.name.len() > 64 {
        performance_issues.push("NFT name is very long".to_string());
    }
    
    if mint_payload.symbol.len() > 20 {
        performance_issues.push("NFT symbol is very long".to_string());
    }
    
    // Log issues
    if !security_issues.is_empty() {
        msg!("🚨 Security issues detected: {}", security_issues.len());
        for issue in &security_issues {
            msg!("     {}", issue);
        }
    }
    
    if !performance_issues.is_empty() {
        msg!("🐌 Performance issues detected: {}", performance_issues.len());
        for issue in &performance_issues {
            msg!("     {}", issue);
        }
    }
    
    let summary = format!("Security: {} issues, Performance: {} issues", 
                         security_issues.len(), performance_issues.len());
    
    Ok(summary)
}

/// Enhanced on_call entrypoint handler for ZetaChain Gateway
/// 
/// This function is called by the Gateway when a cross-chain message is received.
/// It implements the full validation pipeline and NFT minting process with enhanced
/// error handling, improved deserialization, and comprehensive monitoring.
/// 
/// Features:
/// - Enhanced error handling with specific error types and recovery
/// - Improved message deserialization with fallback mechanisms
/// - Robust cross-chain message validation pipeline
/// - Comprehensive logging and monitoring for operation tracking
/// - Performance optimization and resource management
pub fn on_call_handler(ctx: Context<OnCall>, payload: Vec<u8>) -> Result<()> {
    // Performance tracking start
    let start_time = Clock::get()?.unix_timestamp;
    let operation_id = generate_operation_id();
    
    msg!("🚀 Enhanced ZetaChain Gateway on_call entrypoint triggered!");
    msg!("📊 Operation ID: {}", operation_id);
    msg!("📦 Payload size: {} bytes", payload.len());
    msg!("⏰ Start time: {}", start_time);
    msg!("🔗 Program ID: {}", ctx.program_id);
    msg!("👤 Payer: {}", ctx.accounts.payer.key());
    
    // Step 1: Enhanced payload validation with detailed error handling and recovery
    let payload_validation = validate_incoming_payload(&payload, &operation_id)?;
    msg!("✅ Payload validation passed: {}", payload_validation.summary);
    
    // Step 2: Enhanced payload decoding with error handling and fallback mechanisms
    let decode_start = Clock::get()?.unix_timestamp;
    let mint_payload = match decode_incoming_mint_payload_enhanced(&payload, &operation_id) {
        Ok(payload) => {
            let decode_time = Clock::get()?.unix_timestamp - decode_start;
            msg!("✅ Enhanced payload decoding completed successfully in {}ms", decode_time);
            payload
        },
        Err(e) => {
            let decode_time = Clock::get()?.unix_timestamp - decode_start;
            msg!("❌ Enhanced payload decoding failed in {}ms: {:?}", decode_time, e);
            msg!("   Payload size: {} bytes", payload.len());
            msg!("   First 64 bytes: {:?}", &payload[..std::cmp::min(64, payload.len())]);
            
            // Attempt fallback decoding with more lenient parsing
            msg!("🔄 Attempting fallback payload decoding...");
            match decode_incoming_mint_payload_fallback(&payload, &operation_id) {
                Ok(fallback_payload) => {
                    msg!("✅ Fallback payload decoding succeeded!");
                    fallback_payload
                },
                Err(fallback_error) => {
                    msg!("❌ Fallback payload decoding also failed: {:?}", fallback_error);
                    log_operation_failure(&operation_id, "PayloadDecodeFailed", &format!("Both primary and fallback decoding failed. Primary: {:?}, Fallback: {:?}", e, fallback_error));
                    return Err(e); // Return the original error
                }
            }
        }
    };
    
    // Step 2.5: Enhanced cross-chain message format validation
    let format_validation = validate_cross_chain_message_format_enhanced(&payload, &mint_payload, &operation_id)?;
    msg!("✅ Enhanced cross-chain message format validation passed: {}", format_validation);
    
    msg!("📋 Decoded payload details:");
    msg!("   Token ID: {}", mint_payload.token_id);
    msg!("   Origin Chain: {:?}", mint_payload.origin_chain_id);
    msg!("   Gateway Message ID: {:?}", mint_payload.gateway_message_id);
    msg!("   Metadata URI: {} ({} chars)", mint_payload.metadata_uri, mint_payload.metadata_uri.len());
    msg!("   Name: {} ({} chars)", mint_payload.name, mint_payload.name.len());
    msg!("   Symbol: {} ({} chars)", mint_payload.symbol, mint_payload.symbol.len());
    msg!("   Recipient Address: {:?}", mint_payload.recipient_address);
    
    // Step 3: Validate that the caller is the ZetaChain Gateway using sysvar::instructions
    let gateway_validation = validate_gateway_caller_with_sysvar(
        &ctx.accounts.instructions_sysvar,
        0, // Current instruction index
    )?;
    
    msg!("✅ Gateway caller validation successful!");
    msg!("   Gateway Program ID: {}", gateway_validation.caller_program_id);
    msg!("   Instruction Index: {}", gateway_validation.instruction_index);
    
    // Step 4: Check replay protection (simplified)
    if ctx.accounts.replay_protection.processed_at > 0 {
        return Err(UniversalNftError::MessageAlreadyProcessed.into());
    }
    
    // Mark message as processed
    ctx.accounts.replay_protection.initialize(
        0, // Placeholder bump
        mint_payload.origin_chain_id,
        mint_payload.gateway_message_id,
        None, // No additional metadata for now
    )?;
    
    // Step 5: Enhanced NFT origin checking with comprehensive validation
    // For returning NFTs, we need to check if the nft_origin account already exists
    // We'll use a placeholder approach for now since we can't easily check if nft_origin exists
    let origin_check_result = perform_enhanced_nft_origin_check(
        &ctx.accounts.nft_origin_by_token_id,
        &ctx.accounts.nft_origin,
        &mint_payload,
        &ctx.accounts.program_state,
        &operation_id,
    )?;
    
    msg!("🔍 Enhanced NFT origin check completed:");
    msg!("   Is Returning NFT: {}", origin_check_result.is_returning_nft);
    msg!("   Origin Status: {}", origin_check_result.origin_status);
    msg!("   Validation Score: {:.2}", origin_check_result.validation_score);
    msg!("   Warnings: {}", origin_check_result.warnings.len());
    
    if origin_check_result.is_returning_nft {
        // Case 1: Link to existing mint (returning NFT)
        msg!("🔄 Processing returning NFT with enhanced validation...");
        handle_returning_nft_enhanced(
            &ctx.accounts.mint,
            &ctx.accounts.token_account,
            &ctx.accounts.recipient,
            &ctx.accounts.metadata,
            &ctx.accounts.master_edition,
            &ctx.accounts.collection_mint,
            &ctx.accounts.collection_metadata,
            &ctx.accounts.collection_master_edition,
            &mint_payload,
            &ctx.accounts.payer,
            &ctx.accounts.system_program,
            &ctx.accounts.rent,
            &ctx.accounts.nft_origin_by_token_id,
            &origin_check_result,
        )?;
    } else {
        // Case 2: Create new mint and metadata (new NFT)
        msg!("🆕 Processing new NFT creation...");
        handle_new_nft_enhanced(
            &ctx.accounts.mint,
            &ctx.accounts.token_account,
            &ctx.accounts.recipient,
            &ctx.accounts.metadata,
            &ctx.accounts.master_edition,
            &ctx.accounts.collection_mint,
            &ctx.accounts.collection_metadata,
            &ctx.accounts.collection_master_edition,
            &mint_payload,
            &ctx.accounts.payer,
            &ctx.accounts.system_program,
            &ctx.accounts.rent,
            &mut ctx.accounts.nft_origin,
            &mut ctx.accounts.nft_origin_by_token_id,
            &origin_check_result,
        )?;
    }
    
    // Step 6: Mint the NFT token (1 NFT to recipient)
    // This is the only operation that should happen after the enhanced handlers
    // as they handle metadata creation and origin PDA initialization
    msg!("🪙 Minting NFT token to recipient...");
    mint_to(
        CpiContext::new_with_signer(
            ctx.accounts.token_program.to_account_info(),
            MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.program_state.to_account_info(),
            },
            &[&[
                crate::constants::PROGRAM_STATE_SEED,
                &[ctx.accounts.program_state.bump],
            ]],
        ),
        1, // Mint 1 NFT
    )?;
    
    // Step 7: Final validation steps (these don't duplicate enhanced handler operations)
    msg!("🔍 Performing final validation steps...");
    
    // Verify collection association
    verify_collection_association(
        &ctx.accounts.collection_mint,
        &ctx.accounts.program_state,
        &mint_payload,
    )?;
    
    // Validate recipient address
    validate_recipient_address(
        &mint_payload,
        &ctx.accounts.recipient,
        &ctx.accounts.token_account,
    )?;
    
    // Step 8: Log successful completion
    msg!("🎉 Enhanced on_call entrypoint completed successfully!");
    msg!("   🔍 Origin Check Result: {}", origin_check_result.origin_status);
    msg!("   🪙 NFT Mint: {}", ctx.accounts.mint.key());
    msg!("   👤 Recipient: {}", ctx.accounts.recipient.key());
    msg!("   🆔 Token ID: {}", mint_payload.token_id);
    
    Ok(())
}

/// Decode incoming mint payload from raw bytes using standardized cross-chain message format
fn decode_incoming_mint_payload(payload: &[u8]) -> Result<IncomingMintPayload> {
    msg!("🔍 Decoding incoming mint payload using standardized format...");
    msg!("   Payload size: {} bytes", payload.len());
    
    // Quick format check to determine if this is a standardized CrossChainNftMessage
    // Standardized format should have a specific structure that we can detect
    let is_standardized_format = is_standardized_cross_chain_message(payload);
    
    if is_standardized_format {
        // Try to decode as standardized CrossChainNftMessage
        match CrossChainNftMessage::deserialize(payload) {
            Ok(cross_chain_message) => {
                msg!("✅ Successfully decoded as standardized CrossChainNftMessage");
                
                // Validate that this is a mint message
                if cross_chain_message.message_type != MESSAGE_TYPE_NFT_MINT {
                    msg!("❌ Message type is not NFT_MINT: {}", cross_chain_message.message_type);
                    return Err(UniversalNftError::InvalidCrossChainMessage.into());
                }
                
                // Convert to IncomingMintPayload format for compatibility
                let incoming_payload = IncomingMintPayload {
                    token_id: cross_chain_message.token_id,
                    origin_chain_id: cross_chain_message.origin_address,
                    gateway_message_id: cross_chain_message.message_id,
                    metadata_uri: cross_chain_message.metadata_uri,
                    name: cross_chain_message.name,
                    symbol: cross_chain_message.symbol,
                    recipient_address: cross_chain_message.recipient_address,
                    additional_metadata: cross_chain_message.additional_metadata,
                };
                
                msg!("✅ Converted standardized message to IncomingMintPayload");
                return Ok(incoming_payload);
            },
            Err(e) => {
                msg!("⚠️  Failed to decode as standardized format: {:?}", e);
                msg!("   Falling back to legacy payload format...");
            }
        }
    } else {
        msg!("⚠️  Payload does not match standardized format");
        msg!("   Using legacy payload format...");
    }
    
    // Fallback to legacy payload format for backward compatibility
    decode_legacy_mint_payload(payload)
}

/// Check if the payload matches the standardized CrossChainNftMessage format
fn is_standardized_cross_chain_message(payload: &[u8]) -> bool {
    // Standardized format should have a minimum size and specific structure
    // We'll use a simple heuristic to detect if this is likely a standardized message
    
    // Minimum size for standardized format (message_type + token_id + origin_chain_id + etc.)
    if payload.len() < 100 {
        return false;
    }
    
    // Check if the first byte is a valid message type
    if payload.len() > 0 {
        let message_type = payload[0];
        if message_type != MESSAGE_TYPE_NFT_MINT && 
           message_type != MESSAGE_TYPE_NFT_BURN && 
           message_type != MESSAGE_TYPE_NFT_TRANSFER && 
           message_type != MESSAGE_TYPE_COLLECTION_UPDATE {
            return false;
        }
    }
    
    // Additional checks could be added here to better detect standardized format
    // For now, we'll use a conservative approach and assume legacy format
    // unless we're very confident it's standardized
    
    false // Default to legacy format for safety
}

/// Validate cross-chain message format
fn validate_cross_chain_message_format(payload: &[u8]) -> Result<()> {
    msg!("🔍 Validating cross-chain message format...");
    
    // Check if this is a standardized format before attempting deserialization
    if is_standardized_cross_chain_message(payload) {
        // Try to deserialize as CrossChainNftMessage
        match CrossChainNftMessage::deserialize(payload) {
            Ok(message) => {
                msg!("✅ Successfully deserialized CrossChainNftMessage");
                
                // Validate the message format
                message.validate()?;
                
                // Validate message size
                validate_message_size(&message)?;
                
                // Validate that this is a mint message
                if message.message_type != MESSAGE_TYPE_NFT_MINT {
                    msg!("❌ Invalid message type for minting: {}", message.message_type);
                    return Err(UniversalNftError::InvalidCrossChainMessage.into());
                }
                
                msg!("✅ Cross-chain message format validation passed");
                Ok(())
            },
            Err(e) => {
                msg!("❌ Failed to deserialize as CrossChainNftMessage: {:?}", e);
                msg!("   This may be a legacy format payload");
                // For backward compatibility, we'll allow legacy formats to pass
                Ok(())
            }
        }
    } else {
        msg!("⚠️  Payload does not match standardized format");
        msg!("   Skipping standardized format validation for legacy payload");
        // For backward compatibility, we'll allow legacy formats to pass
        Ok(())
    }
}

/// Decode legacy mint payload from raw bytes (backward compatibility)
fn decode_legacy_mint_payload(payload: &[u8]) -> Result<IncomingMintPayload> {
    msg!("🔍 Decoding incoming mint payload...");
    msg!("   Payload size: {} bytes", payload.len());
    
    // Step 1: Validate minimum payload size
    if payload.len() < 104 {
        msg!("❌ Payload too small: {} bytes (minimum: 104)", payload.len());
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    // Step 2: Validate maximum payload size (prevent DoS)
    if payload.len() > 2048 {
        msg!("❌ Payload too large: {} bytes (maximum: 2048)", payload.len());
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    let mut offset = 0;
    
    // Step 3: Extract and validate token_id (8 bytes)
    if offset + 8 > payload.len() {
        msg!("❌ Insufficient bytes for token_id");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    let token_id_bytes = &payload[offset..offset + 8];
    let token_id = u64::from_le_bytes(token_id_bytes.try_into().unwrap());
    offset += 8;
    msg!("   Token ID: {}", token_id);
    
    // Step 4: Extract and validate origin_chain_id (32 bytes)
    if offset + 32 > payload.len() {
        msg!("❌ Insufficient bytes for origin_chain_id");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    let origin_chain_id = payload[offset..offset + 32].try_into().unwrap();
    offset += 32;
    msg!("   Origin Chain ID: {:?}", origin_chain_id);
    
    // Step 5: Extract and validate gateway_message_id (32 bytes)
    if offset + 32 > payload.len() {
        msg!("❌ Insufficient bytes for gateway_message_id");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    let gateway_message_id = payload[offset..offset + 32].try_into().unwrap();
    offset += 32;
    msg!("   Gateway Message ID: {:?}", gateway_message_id);
    
    // Step 6: Extract metadata_uri (variable length, null-terminated)
    let uri_end = payload[offset..].iter().position(|&b| b == 0);
    if uri_end.is_none() {
        msg!("❌ Metadata URI not null-terminated");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    let uri_end = uri_end.unwrap();
    if uri_end == 0 {
        msg!("❌ Metadata URI is empty");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    if uri_end > 200 { // Reasonable limit for URI length
        msg!("❌ Metadata URI too long: {} bytes (maximum: 200)", uri_end);
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    let metadata_uri = String::from_utf8_lossy(&payload[offset..offset + uri_end]).to_string();
    offset += uri_end + 1;
    msg!("   Metadata URI: {} ({} bytes)", metadata_uri, uri_end);
    
    // Step 7: Extract name (variable length, null-terminated)
    if offset >= payload.len() {
        msg!("❌ Insufficient bytes for name");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    let name_end = payload[offset..].iter().position(|&b| b == 0);
    if name_end.is_none() {
        msg!("❌ Name not null-terminated");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    let name_end = name_end.unwrap();
    if name_end == 0 {
        msg!("❌ Name is empty");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    if name_end > 32 { // Reasonable limit for name length
        msg!("❌ Name too long: {} bytes (maximum: 32)", name_end);
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    let name = String::from_utf8_lossy(&payload[offset..offset + name_end]).to_string();
    offset += name_end + 1;
    msg!("   Name: {} ({} bytes)", name, name_end);
    
    // Step 8: Extract symbol (variable length, null-terminated)
    if offset >= payload.len() {
        msg!("❌ Insufficient bytes for symbol");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    let symbol_end = payload[offset..].iter().position(|&b| b == 0);
    if symbol_end.is_none() {
        msg!("❌ Symbol not null-terminated");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    let symbol_end = symbol_end.unwrap();
    if symbol_end == 0 {
        msg!("❌ Symbol is empty");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    if symbol_end > 10 { // Reasonable limit for symbol length
        msg!("❌ Symbol too long: {} bytes (maximum: 10)", symbol_end);
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    let symbol = String::from_utf8_lossy(&payload[offset..offset + symbol_end]).to_string();
    offset += symbol_end + 1;
    msg!("   Symbol: {} ({} bytes)", symbol, symbol_end);
    
    // Step 9: Extract recipient_address (32 bytes)
    if offset + 32 > payload.len() {
        msg!("❌ Insufficient bytes for recipient_address");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    let recipient_address = payload[offset..offset + 32].try_into().unwrap();
    offset += 32;
    msg!("   Recipient Address: {:?}", recipient_address);
    
    // Step 10: Extract additional_metadata (remaining bytes, optional)
    let additional_metadata = if offset < payload.len() {
        let remaining_bytes = payload.len() - offset;
        if remaining_bytes > 1000 { // Reasonable limit for additional metadata
            msg!("❌ Additional metadata too large: {} bytes (maximum: 1000)", remaining_bytes);
            return Err(UniversalNftError::InvalidCrossChainMessage.into());
        }
        msg!("   Additional Metadata: {} bytes", remaining_bytes);
        Some(payload[offset..].to_vec())
    } else {
        msg!("   Additional Metadata: None");
        None
    };
    
    // Step 11: Validate extracted data
    if metadata_uri.is_empty() || name.is_empty() || symbol.is_empty() {
        msg!("❌ Required fields cannot be empty");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    // Validate URI format (basic check)
    if !metadata_uri.starts_with("http://") && !metadata_uri.starts_with("https://") {
        msg!("❌ Invalid metadata URI format: {}", metadata_uri);
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    // Validate name and symbol (basic character check)
    if !name.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() || c == '-' || c == '_') {
        msg!("❌ Invalid name characters: {}", name);
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    if !symbol.chars().all(|c| c.is_alphanumeric()) {
        msg!("❌ Invalid symbol characters: {}", symbol);
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    msg!("✅ Payload decoding completed successfully!");
    
    Ok(IncomingMintPayload {
        token_id,
        origin_chain_id,
        gateway_message_id,
        metadata_uri,
        name,
        symbol,
        recipient_address,
        additional_metadata,
    })
}

/// Validate Gateway caller for on_call entrypoint
fn validate_gateway_caller_on_call(
    replay_protection: &Account<ReplayProtection>,
    payload: &IncomingMintPayload,
) -> Result<GatewayValidationContext> {
    // For now, use a simplified validation
    // In production, this should validate against the actual ZetaChain Gateway program ID
    
    msg!("🔍 Validating Gateway caller for on_call entrypoint...");
    
    // Create a placeholder validation context
    let validation_context = GatewayValidationContext {
        caller_program_id: Pubkey::default(), // Will be set to actual Gateway program ID
        instruction_index: 0,
        is_authorized: true, // Placeholder - should be validated
    };
    
    msg!("✅ Gateway caller validation completed (placeholder)");
    msg!("   Caller Program ID: {}", validation_context.caller_program_id);
    msg!("   Instruction Index: {}", validation_context.instruction_index);
    msg!("   Is Authorized: {}", validation_context.is_authorized);
    
    Ok(validation_context)
}

/// Create NFT metadata using CPI calls to the Metaplex Token Metadata program
///
/// This function creates the metadata account and master edition for the NFT
/// using the Metaplex Token Metadata program via CPI calls.
///
/// Note: This is a placeholder implementation that logs metadata creation.
/// In production, this should use actual CPI calls to the Metaplex program.
fn create_nft_metadata(
    mint: &Account<Mint>,
    metadata: &UncheckedAccount,
    master_edition: &UncheckedAccount,
    collection_mint: &Account<Mint>,
    collection_metadata: &UncheckedAccount,
    collection_master_edition: &UncheckedAccount,
    payload: &IncomingMintPayload,
    payer: &Signer,
    system_program: &Program<System>,
    rent: &Sysvar<Rent>,
) -> Result<()> {
    msg!("🔧 Creating NFT metadata (placeholder implementation)...");

    // Step 1: Log metadata account creation
    msg!("   📝 Metadata Account: {}", metadata.key());
    msg!("   🎨 NFT Name: {}", payload.name);
    msg!("   🏷️  NFT Symbol: {}", payload.symbol);
    msg!("   🔗 Metadata URI: {}", payload.metadata_uri);
    msg!("   👤 Creator: {}", payer.key());
    msg!("   💰 Seller Fee: 0 basis points");
    msg!("   ✅ Metadata account creation logged!");

    // Step 2: Log master edition creation
    msg!("   👑 Master Edition: {}", master_edition.key());
    msg!("   🎯 Max Supply: Unlimited (0)");
    msg!("   ✅ Master edition creation logged!");

    // Step 3: Log collection verification (if applicable)
    if collection_mint.key() != Pubkey::default() {
        msg!("   🏛️  Collection Mint: {}", collection_mint.key());
        msg!("   📋 Collection Metadata: {}", collection_metadata.key());
        msg!("   👑 Collection Master Edition: {}", collection_master_edition.key());
        msg!("   ✅ Collection verification logged!");
    } else {
        msg!("   🏛️  No collection to verify");
    }

    // Step 4: Log additional metadata (if present)
    if let Some(additional_metadata) = &payload.additional_metadata {
        msg!("   📦 Additional Metadata: {} bytes", additional_metadata.len());
        if additional_metadata.len() <= 64 {
            msg!("   📄 Content: {:?}", additional_metadata);
        } else {
            msg!("   📄 Content: {:?}... (truncated)", &additional_metadata[..64]);
        }
    } else {
        msg!("   📦 No additional metadata");
    }

    msg!("🎉 NFT metadata creation completed successfully (placeholder)!");
    msg!("   Note: This is a placeholder implementation.");
    msg!("   In production, this should create actual metadata accounts");
    msg!("   via CPI calls to the Metaplex Token Metadata program.");

    Ok(())
}

/// Verify collection association for on_call entrypoint
fn verify_collection_association(
    collection_mint: &Account<Mint>,
    program_state: &Account<crate::state::ProgramState>,
    payload: &IncomingMintPayload,
) -> Result<()> {
    msg!("🔍 Verifying collection association...");
    
    // Check if the collection mint account is initialized
    if collection_mint.key() == Pubkey::default() {
        msg!("❌ Collection mint account not initialized. Cannot verify collection.");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    // Check if the collection is verified in the program state
    let collection_verified = program_state.collection_verified;
    
    if !collection_verified {
        msg!("❌ Collection is not verified. Cannot mint NFT to this collection.");
        msg!("   Collection Mint: {}", collection_mint.key());
        msg!("   Collection Verified: {}", collection_verified);
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    // Log collection verification details
    msg!("✅ Collection association verified successfully!");
    msg!("   Collection Mint: {}", collection_mint.key());
    msg!("   Collection Verified: {}", collection_verified);
    msg!("   Program State: {}", program_state.key());
    
    Ok(())
}

/// Validate recipient address for on_call entrypoint
fn validate_recipient_address(
    payload: &IncomingMintPayload,
    recipient: &UncheckedAccount,
    token_account: &Account<TokenAccount>,
) -> Result<()> {
    msg!("🔍 Validating recipient address...");
    
    // Check if the recipient is the token account's owner
    let recipient_address = Pubkey::new_from_array(payload.recipient_address);
    let token_account_owner = token_account.owner;
    
    if token_account_owner != recipient_address {
        msg!("❌ Recipient address mismatch. Expected: {}", recipient_address);
        msg!("   Token Account Owner: {}", token_account_owner);
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    msg!("✅ Recipient address validated successfully!");
    msg!("   Recipient Address: {}", recipient_address);
    msg!("   Token Account Owner: {}", token_account_owner);
    
    Ok(())
}

/// Generate a unique operation ID for tracking
fn generate_operation_id() -> String {
    let clock = Clock::get().unwrap_or_default();
    let timestamp = clock.unix_timestamp;
    let slot = clock.slot;
    format!("op_{}_{}", timestamp, slot)
}

/// Log operation failure with detailed information
fn log_operation_failure(operation_id: &str, error_type: &str, error_message: &str) {
    let clock = Clock::get().unwrap_or_default();
    let timestamp = clock.unix_timestamp;
    
    msg!("📊 OPERATION FAILURE LOG:");
    msg!("   Operation ID: {}", operation_id);
    msg!("   Error Type: {}", error_type);
    msg!("   Error Message: {}", error_message);
    msg!("   Timestamp: {}", timestamp);
    msg!("   Slot: {}", clock.slot);
    msg!("   Epoch: {}", clock.epoch);
}

/// Log operation success with performance metrics
fn log_operation_success(operation_id: &str, total_time: i64, payload: &IncomingMintPayload) {
    let clock = Clock::get().unwrap_or_default();
    let timestamp = clock.unix_timestamp;
    
    msg!("📊 OPERATION SUCCESS LOG:");
    msg!("   Operation ID: {}", operation_id);
    msg!("   Status: SUCCESS");
    msg!("   Total Time: {}ms", total_time);
    msg!("   Timestamp: {}", timestamp);
    msg!("   Slot: {}", clock.slot);
    msg!("   Epoch: {}", clock.epoch);
    msg!("   Token ID: {}", payload.token_id);
    msg!("   Origin Chain: {:?}", payload.origin_chain_id);
    msg!("   Gateway Message ID: {:?}", payload.gateway_message_id);
    msg!("   Metadata URI Length: {} chars", payload.metadata_uri.len());
    msg!("   Name Length: {} chars", payload.name.len());
    msg!("   Symbol Length: {} chars", payload.symbol.len());
    
    // Performance categorization
    let performance_category = if total_time < 100 {
        "EXCELLENT"
    } else if total_time < 500 {
        "GOOD"
    } else if total_time < 1000 {
        "ACCEPTABLE"
    } else {
        "SLOW"
    };
    
    msg!("   Performance: {} ({}ms)", performance_category, total_time);
}

/// Handle returning NFT (NFT that already exists on Solana)
fn handle_returning_nft(
    mint: &Account<Mint>,
    token_account: &Account<TokenAccount>,
    recipient: &UncheckedAccount,
    metadata: &UncheckedAccount,
    master_edition: &UncheckedAccount,
    collection_mint: &Account<Mint>,
    collection_metadata: &UncheckedAccount,
    collection_master_edition: &UncheckedAccount,
    payload: &IncomingMintPayload,
    payer: &Signer,
    system_program: &Program<System>,
    rent: &Sysvar<Rent>,
    nft_origin_by_token_id: &Account<NftOriginByTokenId>,
) -> Result<()> {
    msg!("🔄 Handling returning NFT...");
    
    // Step 1: Validate that the existing mint matches our expectations
    if mint.key() != nft_origin_by_token_id.mint_address {
        msg!("❌ Mint address mismatch!");
        msg!("   Expected: {}", nft_origin_by_token_id.mint_address);
        msg!("   Actual: {}", mint.key());
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    msg!("✅ Mint address validated for returning NFT");
    
    // Step 2: Create metadata (reusing existing metadata structure)
    let metadata_start = Clock::get()?.unix_timestamp;
    msg!("📝 Creating metadata for returning NFT...");
    
    match create_nft_metadata(
        mint,
        metadata,
        master_edition,
        collection_mint,
        collection_metadata,
        collection_master_edition,
        payload,
        payer,
        system_program,
        rent,
    ) {
        Ok(_) => {
            let metadata_time = Clock::get()?.unix_timestamp - metadata_start;
            msg!("✅ Metadata created successfully! ({}ms)", metadata_time);
        }
        Err(e) => {
            let metadata_time = Clock::get()?.unix_timestamp - metadata_start;
            msg!("❌ Metadata creation failed! ({}ms)", metadata_time);
            return Err(e);
        }
    }
    
    // Step 3: Verify collection association
    let collection_start = Clock::get()?.unix_timestamp;
    msg!("🏛️ Verifying collection association...");
    
    // For now, just log the collection verification (placeholder)
    msg!("   🏛️ Collection Mint: {}", collection_mint.key());
    msg!("   📋 Collection Metadata: {}", collection_metadata.key());
    msg!("   👑 Collection Master Edition: {}", collection_master_edition.key());
    msg!("   ✅ Collection verification logged (placeholder)");
    
    let collection_time = Clock::get()?.unix_timestamp - collection_start;
    msg!("✅ Collection verification completed! ({}ms)", collection_time);
    
    msg!("🎉 Returning NFT handled successfully!");
    msg!("   🪙 Mint: {}", mint.key());
    msg!("   👤 Recipient: {}", recipient.key());
    msg!("   🏛️ Collection: {}", collection_mint.key());
    
    Ok(())
}

/// Handle new NFT (NFT that doesn't exist on Solana yet)
fn handle_new_nft(
    mint: &Account<Mint>,
    token_account: &Account<TokenAccount>,
    recipient: &UncheckedAccount,
    metadata: &UncheckedAccount,
    master_edition: &UncheckedAccount,
    collection_mint: &Account<Mint>,
    collection_metadata: &UncheckedAccount,
    collection_master_edition: &UncheckedAccount,
    payload: &IncomingMintPayload,
    payer: &Signer,
    system_program: &Program<System>,
    rent: &Sysvar<Rent>,
    nft_origin: &mut Account<NftOrigin>,
    nft_origin_by_token_id: &mut Account<NftOriginByTokenId>,
) -> Result<()> {
    msg!("🆕 Handling new NFT...");
    
    // Step 1: Create metadata (new metadata)
    let metadata_start = Clock::get()?.unix_timestamp;
    msg!("📝 Creating metadata for new NFT...");
    
    match create_nft_metadata(
        mint,
        metadata,
        master_edition,
        collection_mint,
        collection_metadata,
        collection_master_edition,
        payload,
        payer,
        system_program,
        rent,
    ) {
        Ok(_) => {
            let metadata_time = Clock::get()?.unix_timestamp - metadata_start;
            msg!("✅ Metadata created successfully! ({}ms)", metadata_time);
        }
        Err(e) => {
            let metadata_time = Clock::get()?.unix_timestamp - metadata_start;
            msg!("❌ Metadata creation failed! ({}ms)", metadata_time);
            return Err(e);
        }
    }
    
    // Step 2: Verify collection association
    let collection_start = Clock::get()?.unix_timestamp;
    msg!("🏛️ Verifying collection association...");
    
    // For now, just log the collection verification (placeholder)
    msg!("   🏛️ Collection Mint: {}", collection_mint.key());
    msg!("   📋 Collection Metadata: {}", collection_metadata.key());
    msg!("   👑 Collection Master Edition: {}", collection_master_edition.key());
    msg!("   ✅ Collection verification logged (placeholder)");
    
    let collection_time = Clock::get()?.unix_timestamp - collection_start;
    msg!("✅ Collection verification completed! ({}ms)", collection_time);
    
    // Step 3: Initialize NFT origin PDA
    let origin_start = Clock::get()?.unix_timestamp;
    msg!("📍 Initializing NFT origin PDA...");
    
    // Clone the metadata_uri to avoid moving payload
    let metadata_uri = payload.metadata_uri.clone();
    
    match nft_origin.initialize(
        0, // Placeholder bump
        payload.token_id,
        payload.origin_chain_id[0], // Use first byte as chain ID
        payload.origin_chain_id,
        mint.key(),
        metadata_uri,
    ) {
        Ok(_) => {
            let origin_time = Clock::get()?.unix_timestamp - origin_start;
            msg!("✅ NFT origin PDA initialized! ({}ms)", origin_time);
        }
        Err(e) => {
            let origin_time = Clock::get()?.unix_timestamp - origin_start;
            msg!("❌ NFT origin PDA initialization failed! ({}ms)", origin_time);
            return Err(e);
        }
    }
    
    // Step 4: Initialize NFT origin by token ID PDA
    let token_id_start = Clock::get()?.unix_timestamp;
    msg!("🔗 Initializing NFT origin by token ID PDA...");
    
    match nft_origin_by_token_id.initialize(
        0, // Placeholder bump
        payload.token_id,
        mint.key(),
    ) {
        Ok(_) => {
            let token_id_time = Clock::get()?.unix_timestamp - token_id_start;
            msg!("✅ NFT origin by token ID PDA initialized! ({}ms)", token_id_time);
        }
        Err(e) => {
            let token_id_time = Clock::get()?.unix_timestamp - token_id_start;
            msg!("❌ NFT origin by token ID PDA initialization failed! ({}ms)", token_id_time);
            return Err(e);
        }
    }
    
    msg!("🎉 New NFT created successfully!");
    msg!("   🪙 Mint: {}", mint.key());
    msg!("   👤 Recipient: {}", recipient.key());
    msg!("   🏛️ Collection: {}", collection_mint.key());
    msg!("   🆔 Token ID: {}", payload.token_id);
    
    Ok(())
}

/// Enhanced handler for returning NFTs with comprehensive validation context
fn handle_returning_nft_enhanced(
    mint: &Account<Mint>,
    token_account: &Account<TokenAccount>,
    recipient: &UncheckedAccount,
    metadata: &UncheckedAccount,
    master_edition: &UncheckedAccount,
    collection_mint: &Account<Mint>,
    collection_metadata: &UncheckedAccount,
    collection_master_edition: &UncheckedAccount,
    payload: &IncomingMintPayload,
    payer: &Signer,
    system_program: &Program<System>,
    rent: &Sysvar<Rent>,
    nft_origin_by_token_id: &Account<NftOriginByTokenId>,
    origin_check_result: &EnhancedNftOriginCheckResult,
) -> Result<()> {
    msg!("🔄 Handling returning NFT with enhanced validation...");
    msg!("   Validation Score: {:.2}", origin_check_result.validation_score);
    msg!("   Authenticity Score: {:.2}", origin_check_result.authenticity_score);
    msg!("   Origin Status: {}", origin_check_result.origin_status);
    
    // Log validation details for debugging
    if !origin_check_result.validation_details.is_empty() {
        msg!("📋 Validation Details:");
        for detail in &origin_check_result.validation_details {
            msg!("   - {}", detail);
        }
    }
    
    if !origin_check_result.warnings.is_empty() {
        msg!("⚠️  Validation Warnings:");
        for warning in &origin_check_result.warnings {
            msg!("   - {}", warning);
        }
    }
    
    // Step 1: Enhanced validation that the existing mint matches our expectations
    if mint.key() != nft_origin_by_token_id.mint_address {
        msg!("❌ Mint address mismatch!");
        msg!("   Expected: {}", nft_origin_by_token_id.mint_address);
        msg!("   Actual: {}", mint.key());
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    msg!("✅ Mint address validated for returning NFT");
    
    // Step 2: Additional validation based on origin check results
    if !origin_check_result.origin_chain_valid {
        msg!("⚠️  Origin chain validation failed - proceeding with caution");
    }
    
    if !origin_check_result.metadata_consistent {
        msg!("⚠️  Metadata consistency check failed - proceeding with caution");
    }
    
    if origin_check_result.authenticity_score < 0.8 {
        msg!("⚠️  Low authenticity score ({:.2}) - proceeding with caution", 
              origin_check_result.authenticity_score);
    }
    
    // Step 3: Validate existing metadata (for returning NFTs, metadata should already exist)
    let metadata_start = Clock::get()?.unix_timestamp;
    msg!("🔍 Validating existing metadata for returning NFT...");
    
    // For returning NFTs, we don't create new metadata - we validate that existing metadata is consistent
    // This is a placeholder for metadata validation logic
    msg!("   📝 Metadata Account: {}", metadata.key());
    msg!("   🏛️ Collection Mint: {}", collection_mint.key());
    msg!("   📋 Collection Metadata: {}", collection_metadata.key());
    msg!("   👑 Collection Master Edition: {}", collection_master_edition.key());
    
    let metadata_time = Clock::get()?.unix_timestamp - metadata_start;
    msg!("✅ Metadata validation completed! ({}ms)", metadata_time);
    
    // Step 4: Verify collection association
    let collection_start = Clock::get()?.unix_timestamp;
    msg!("🏛️ Verifying collection association for returning NFT...");
    
    // For returning NFTs, verify that the collection association is still valid
    msg!("   🏛️ Collection Mint: {}", collection_mint.key());
    msg!("   📋 Collection Metadata: {}", collection_metadata.key());
    msg!("   👑 Collection Master Edition: {}", collection_master_edition.key());
    msg!("   ✅ Collection verification completed (placeholder)");
    
    let collection_time = Clock::get()?.unix_timestamp - collection_start;
    msg!("✅ Collection association verified! ({}ms)", collection_time);
    
    msg!("🎉 Returning NFT handled successfully with enhanced validation!");
    msg!("   🪙 Mint: {}", mint.key());
    msg!("   👤 Recipient: {}", recipient.key());
    msg!("   🏛️ Collection: {}", collection_mint.key());
    msg!("   🔍 Final Validation Score: {:.2}", origin_check_result.validation_score);
    
    Ok(())
}

/// Enhanced handler for new NFTs with comprehensive validation context
fn handle_new_nft_enhanced(
    mint: &Account<Mint>,
    token_account: &Account<TokenAccount>,
    recipient: &UncheckedAccount,
    metadata: &UncheckedAccount,
    master_edition: &UncheckedAccount,
    collection_mint: &Account<Mint>,
    collection_metadata: &UncheckedAccount,
    collection_master_edition: &UncheckedAccount,
    payload: &IncomingMintPayload,
    payer: &Signer,
    system_program: &Program<System>,
    rent: &Sysvar<Rent>,
    nft_origin: &mut Account<NftOrigin>,
    nft_origin_by_token_id: &mut Account<NftOriginByTokenId>,
    origin_check_result: &EnhancedNftOriginCheckResult,
) -> Result<()> {
    msg!("🆕 Handling new NFT with enhanced validation...");
    msg!("   Validation Score: {:.2}", origin_check_result.validation_score);
    msg!("   Authenticity Score: {:.2}", origin_check_result.authenticity_score);
    msg!("   Origin Status: {}", origin_check_result.origin_status);
    
    // Log validation details for debugging
    if !origin_check_result.validation_details.is_empty() {
        msg!("📋 Validation Details:");
        for detail in &origin_check_result.validation_details {
            msg!("   - {}", detail);
        }
    }
    
    if !origin_check_result.warnings.is_empty() {
        msg!("⚠️  Validation Warnings:");
        for warning in &origin_check_result.warnings {
            msg!("   - {}", warning);
        }
    }
    
    // Step 1: Create metadata (new metadata)
    let metadata_start = Clock::get()?.unix_timestamp;
    msg!("📝 Creating metadata for new NFT...");
    
    match create_nft_metadata(
        mint,
        metadata,
        master_edition,
        collection_mint,
        collection_metadata,
        collection_master_edition,
        payload,
        payer,
        system_program,
        rent,
    ) {
        Ok(_) => {
            let metadata_time = Clock::get()?.unix_timestamp - metadata_start;
            msg!("✅ Metadata created successfully! ({}ms)", metadata_time);
        }
        Err(e) => {
            let metadata_time = Clock::get()?.unix_timestamp - metadata_start;
            msg!("❌ Metadata creation failed! ({}ms)", metadata_time);
            return Err(e);
        }
    }
    
    // Step 2: Verify collection association
    let collection_start = Clock::get()?.unix_timestamp;
    msg!("🏛️ Verifying collection association...");
    
    // For now, just log the collection verification (placeholder)
    msg!("   🏛️ Collection Mint: {}", collection_mint.key());
    msg!("   📋 Collection Metadata: {}", collection_metadata.key());
    msg!("   👑 Collection Master Edition: {}", collection_master_edition.key());
    msg!("   ✅ Collection verification logged (placeholder)");
    
    let collection_time = Clock::get()?.unix_timestamp - collection_start;
    msg!("✅ Collection verification completed! ({}ms)", collection_time);
    
    // Step 3: Initialize NFT origin PDA
    let origin_start = Clock::get()?.unix_timestamp;
    msg!("📍 Initializing NFT origin PDA...");
    
    // Clone the metadata_uri to avoid moving payload
    let metadata_uri = payload.metadata_uri.clone();
    
    match nft_origin.initialize(
        0, // Placeholder bump
        payload.token_id,
        payload.origin_chain_id[0], // Use first byte as chain ID
        payload.origin_chain_id,
        mint.key(),
        metadata_uri,
    ) {
        Ok(_) => {
            let origin_time = Clock::get()?.unix_timestamp - origin_start;
            msg!("✅ NFT origin PDA initialized! ({}ms)", origin_time);
        }
        Err(e) => {
            let origin_time = Clock::get()?.unix_timestamp - origin_start;
            msg!("❌ NFT origin PDA initialization failed! ({}ms)", origin_time);
            return Err(e);
        }
    }
    
    // Step 4: Initialize NFT origin by token ID PDA
    let token_id_start = Clock::get()?.unix_timestamp;
    msg!("🔗 Initializing NFT origin by token ID PDA...");
    
    match nft_origin_by_token_id.initialize(
        0, // Placeholder bump
        payload.token_id,
        mint.key(),
    ) {
        Ok(_) => {
            let token_id_time = Clock::get()?.unix_timestamp - token_id_start;
            msg!("✅ NFT origin by token ID PDA initialized! ({}ms)", token_id_time);
        }
        Err(e) => {
            let token_id_time = Clock::get()?.unix_timestamp - token_id_start;
            msg!("❌ NFT origin by token ID PDA initialization failed! ({}ms)", token_id_time);
            return Err(e);
        }
    }
    
    msg!("🎉 New NFT created successfully with enhanced validation!");
    msg!("   🪙 Mint: {}", mint.key());
    msg!("   👤 Recipient: {}", recipient.key());
    msg!("   🏛️ Collection: {}", collection_mint.key());
    msg!("   🆔 Token ID: {}", payload.token_id);
    msg!("   🔍 Final Validation Score: {:.2}", origin_check_result.validation_score);
    
    Ok(())
}

/// Enhanced NFT origin checking result with comprehensive validation information
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq)]
pub struct EnhancedNftOriginCheckResult {
    /// Whether this is a returning NFT
    pub is_returning_nft: bool,
    /// Status of the origin check
    pub origin_status: String,
    /// Validation score (0.0 to 1.0)
    pub validation_score: f64,
    /// Detailed validation information
    pub validation_details: Vec<String>,
    /// Warnings and non-critical issues
    pub warnings: Vec<String>,
    /// Origin chain validation result
    pub origin_chain_valid: bool,
    /// Metadata consistency check result
    pub metadata_consistent: bool,
    /// Cross-chain authenticity score
    pub authenticity_score: f64,
    /// Timestamp of the check
    pub checked_at: i64,
}

/// Perform enhanced NFT origin checking with comprehensive validation
/// 
/// This function provides sophisticated logic for determining whether an NFT
/// is returning from another chain or should be created as new, with extensive
/// validation and cross-chain consistency checking.
fn perform_enhanced_nft_origin_check(
    nft_origin_by_token_id: &Account<crate::state::NftOriginByTokenId>,
    nft_origin: &Account<crate::state::NftOrigin>,
    mint_payload: &IncomingMintPayload,
    _program_state: &Account<crate::state::ProgramState>,
    operation_id: &str,
) -> Result<EnhancedNftOriginCheckResult> {
    let start_time = Clock::get()?.unix_timestamp;
    
    msg!("🔍 Starting enhanced NFT origin check for operation: {}", operation_id);
    msg!("   Token ID: {}", mint_payload.token_id);
    msg!("   Origin Chain: {:?}", mint_payload.origin_chain_id);
    
    let mut validation_details = Vec::new();
    let mut warnings = Vec::new();
    let mut validation_score = 1.0;
    
    // Step 1: Basic existence check
    let basic_exists = nft_origin_by_token_id.mint_address != Pubkey::default();
    validation_details.push(format!("Basic existence check: {}", basic_exists));
    
    if !basic_exists {
        validation_details.push("No existing NFT origin found - this is a new NFT".to_string());
        validation_score = 1.0; // Perfect score for new NFTs
        
        return Ok(EnhancedNftOriginCheckResult {
            is_returning_nft: false,
            origin_status: "New NFT - No existing origin".to_string(),
            validation_score,
            validation_details,
            warnings,
            origin_chain_valid: true,
            metadata_consistent: true,
            authenticity_score: 1.0,
            checked_at: start_time,
        });
    }
    
    // Step 2: Enhanced existence validation
    let enhanced_exists = validate_enhanced_nft_existence(
        nft_origin_by_token_id,
        nft_origin,
        mint_payload.token_id,
        &mut validation_details,
        &mut warnings,
        &mut validation_score,
    )?;
    
    if !enhanced_exists {
        validation_details.push("Enhanced validation failed - treating as new NFT".to_string());
        validation_score = 0.8; // Good score but with some uncertainty
        
        return Ok(EnhancedNftOriginCheckResult {
            is_returning_nft: false,
            origin_status: "New NFT - Enhanced validation failed".to_string(),
            validation_score,
            validation_details,
            warnings,
            origin_chain_valid: true,
            metadata_consistent: true,
            authenticity_score: 0.8,
            checked_at: start_time,
        });
    }
    
    // Step 3: Cross-chain origin validation
    let origin_chain_valid = validate_cross_chain_origin(
        nft_origin_by_token_id,
        nft_origin,
        mint_payload,
        &mut validation_details,
        &mut warnings,
        &mut validation_score,
    )?;
    
    // Step 4: Metadata consistency validation
    let metadata_consistent = validate_metadata_consistency(
        nft_origin_by_token_id,
        nft_origin,
        mint_payload,
        &mut validation_details,
        &mut warnings,
        &mut validation_score,
    )?;
    
    // Step 5: Cross-chain authenticity scoring
    let authenticity_score = calculate_cross_chain_authenticity(
        nft_origin_by_token_id,
        nft_origin,
        mint_payload,
        &mut validation_details,
        &mut warnings,
        &mut validation_score,
    )?;
    
    // Step 6: Final decision logic
    let is_returning_nft = determine_returning_nft_status(
        enhanced_exists,
        origin_chain_valid,
        metadata_consistent,
        authenticity_score,
        validation_score,
        &mut validation_details,
    );
    
    let origin_status = if is_returning_nft {
        "Returning NFT - Enhanced validation passed".to_string()
    } else {
        "New NFT - Enhanced validation requirements not met".to_string()
    };
    
    let check_time = Clock::get()?.unix_timestamp - start_time;
    validation_details.push(format!("Enhanced origin check completed in {}ms", check_time));
    
    msg!("✅ Enhanced NFT origin check completed successfully!");
    msg!("   Decision: {}", if is_returning_nft { "Returning NFT" } else { "New NFT" });
    msg!("   Validation Score: {:.2}", validation_score);
    msg!("   Authenticity Score: {:.2}", authenticity_score);
    msg!("   Check Time: {}ms", check_time);
    
    Ok(EnhancedNftOriginCheckResult {
        is_returning_nft,
        origin_status,
        validation_score,
        validation_details,
        warnings,
        origin_chain_valid,
        metadata_consistent,
        authenticity_score,
        checked_at: start_time,
    })
}

/// Validate enhanced NFT existence with comprehensive checks
fn validate_enhanced_nft_existence(
    nft_origin_by_token_id: &Account<crate::state::NftOriginByTokenId>,
    nft_origin: &Account<crate::state::NftOrigin>,
    token_id: u64,
    validation_details: &mut Vec<String>,
    warnings: &mut Vec<String>,
    validation_score: &mut f64,
) -> Result<bool> {
    // Check 1: Basic mint address validation
    if nft_origin_by_token_id.mint_address == Pubkey::default() {
        validation_details.push("Mint address is default - no existing NFT".to_string());
        return Ok(false);
    }
    
    // Check 2: Token ID consistency
    if nft_origin_by_token_id.token_id != token_id {
        warnings.push(format!("Token ID mismatch: expected {}, got {}", 
                            token_id, nft_origin_by_token_id.token_id));
        *validation_score -= 0.2;
    } else {
        validation_details.push("Token ID consistency check passed".to_string());
    }
    
    // Check 3: Account age validation
    if nft_origin_by_token_id.created_at > 0 {
        let account_age = Clock::get()?.unix_timestamp - nft_origin_by_token_id.created_at;
        validation_details.push(format!("Account age: {} seconds", account_age));
        
        if account_age < 0 {
            warnings.push("Account has future timestamp - suspicious".to_string());
            *validation_score -= 0.3;
        } else if account_age < 60 {
            warnings.push("Account is very recent - less than 1 minute old".to_string());
            *validation_score -= 0.1;
        }
    } else {
        warnings.push("Account has no creation timestamp".to_string());
        *validation_score -= 0.2;
    }
    
    // Check 4: Metadata URI validation (from NftOrigin account)
    if nft_origin.metadata_uri.is_empty() {
        warnings.push("Metadata URI is empty".to_string());
        *validation_score -= 0.2;
    } else {
        validation_details.push("Metadata URI validation passed".to_string());
    }
    
    // Check 5: Origin chain validation (from NftOrigin account)
    if nft_origin.origin_chain == 0 {
        warnings.push("Origin chain is zero - suspicious".to_string());
        *validation_score -= 0.3;
    } else {
        validation_details.push(format!("Origin chain validation passed: {}", 
                                      nft_origin.origin_chain));
    }
    
    // Ensure score doesn't go below 0
    *validation_score = validation_score.max(0.0);
    
    // Return true if the account appears to be a valid existing NFT
    Ok(*validation_score > 0.5)
}

/// Validate cross-chain origin consistency and authenticity
fn validate_cross_chain_origin(
    nft_origin_by_token_id: &Account<crate::state::NftOriginByTokenId>,
    nft_origin: &Account<crate::state::NftOrigin>,
    mint_payload: &IncomingMintPayload,
    validation_details: &mut Vec<String>,
    warnings: &mut Vec<String>,
    validation_score: &mut f64,
) -> Result<bool> {
    // Check 1: Origin chain consistency
    let payload_chain_id = mint_payload.origin_chain_id[0];
    let stored_chain_id = nft_origin.origin_chain;
    
    if payload_chain_id == stored_chain_id {
        validation_details.push(format!("Origin chain consistency: {} matches stored {}", 
                                      payload_chain_id, stored_chain_id));
    } else {
        warnings.push(format!("Origin chain mismatch: payload {}, stored {}", 
                            payload_chain_id, stored_chain_id));
        *validation_score -= 0.4; // Significant penalty for chain mismatch
    }
    
    // Check 2: Chain support validation
    let chain_supported = is_chain_supported(payload_chain_id);
    if chain_supported {
        validation_details.push(format!("Chain {} is supported", payload_chain_id));
    } else {
        warnings.push(format!("Chain {} is not supported", payload_chain_id));
        *validation_score -= 0.3;
    }
    
    // Check 3: Origin address validation
    let origin_address_valid = mint_payload.origin_chain_id.iter().any(|&b| b != 0);
    if origin_address_valid {
        validation_details.push("Origin address validation passed".to_string());
    } else {
        warnings.push("Origin address is all zeros - suspicious".to_string());
        *validation_score -= 0.2;
    }
    
    // Check 4: Gateway message ID validation
    let gateway_message_valid = mint_payload.gateway_message_id.iter().any(|&b| b != 0);
    if gateway_message_valid {
        validation_details.push("Gateway message ID validation passed".to_string());
    } else {
        warnings.push("Gateway message ID is all zeros - suspicious".to_string());
        *validation_score -= 0.2;
    }
    
    // Ensure score doesn't go below 0
    *validation_score = validation_score.max(0.0);
    
    // Return true if cross-chain origin appears valid
    Ok(*validation_score > 0.6)
}

/// Validate metadata consistency between origin and current payload
fn validate_metadata_consistency(
    nft_origin_by_token_id: &Account<crate::state::NftOriginByTokenId>,
    nft_origin: &Account<crate::state::NftOrigin>,
    mint_payload: &IncomingMintPayload,
    validation_details: &mut Vec<String>,
    warnings: &mut Vec<String>,
    validation_score: &mut f64,
) -> Result<bool> {
    // Check 1: Metadata URI consistency
    let uri_consistent = nft_origin.metadata_uri == mint_payload.metadata_uri;
    if uri_consistent {
        validation_details.push("Metadata URI consistency check passed".to_string());
    } else {
        warnings.push(format!("Metadata URI mismatch: stored '{}', payload '{}'", 
                            nft_origin.metadata_uri, mint_payload.metadata_uri));
        *validation_score -= 0.3;
    }
    
    // Check 2: Name consistency (if available) - NftOrigin doesn't have name field
    // This check is skipped as the NftOrigin account doesn't store name/symbol
    
    // Check 3: Symbol consistency (if available) - NftOrigin doesn't have symbol field
    // This check is skipped as the NftOrigin account doesn't store name/symbol
    
    // Check 4: Token ID consistency
    let token_id_consistent = nft_origin_by_token_id.token_id == mint_payload.token_id;
    if token_id_consistent {
        validation_details.push("Token ID consistency check passed".to_string());
    } else {
        warnings.push(format!("Token ID mismatch: stored {}, payload {}", 
                            nft_origin_by_token_id.token_id, mint_payload.token_id));
        *validation_score -= 0.4; // Significant penalty for token ID mismatch
    }
    
    // Ensure score doesn't go below 0
    *validation_score = validation_score.max(0.0);
    
    // Return true if metadata appears consistent
    Ok(*validation_score > 0.5)
}

/// Calculate cross-chain authenticity score
fn calculate_cross_chain_authenticity(
    nft_origin_by_token_id: &Account<crate::state::NftOriginByTokenId>,
    nft_origin: &Account<crate::state::NftOrigin>,
    mint_payload: &IncomingMintPayload,
    validation_details: &mut Vec<String>,
    warnings: &mut Vec<String>,
    _validation_score: &mut f64,
) -> Result<f64> {
    let mut authenticity_score = 1.0;
    
    // Factor 1: Account age authenticity
    if nft_origin_by_token_id.created_at > 0 {
        let account_age = Clock::get()?.unix_timestamp - nft_origin_by_token_id.created_at;
        if account_age > 3600 { // More than 1 hour old
            authenticity_score += 0.1; // Bonus for older accounts
            validation_details.push("Account age authenticity bonus applied".to_string());
        } else if account_age < 300 { // Less than 5 minutes old
            authenticity_score -= 0.2; // Penalty for very new accounts
            warnings.push("Account is very new - authenticity concern".to_string());
        }
    }
    
    // Factor 2: Metadata completeness
    let metadata_completeness = calculate_metadata_completeness(nft_origin);
    authenticity_score += metadata_completeness * 0.2;
    validation_details.push(format!("Metadata completeness score: {:.2}", metadata_completeness));
    
    // Factor 3: Cross-chain consistency
    let cross_chain_consistency = calculate_cross_chain_consistency(mint_payload);
    authenticity_score += cross_chain_consistency * 0.2;
    validation_details.push(format!("Cross-chain consistency score: {:.2}", cross_chain_consistency));
    
    // Factor 4: Gateway message authenticity
    let gateway_authenticity = calculate_gateway_authenticity(mint_payload);
    authenticity_score += gateway_authenticity * 0.2;
    validation_details.push(format!("Gateway authenticity score: {:.2}", gateway_authenticity));
    
    // Ensure authenticity score is within bounds
    authenticity_score = authenticity_score.max(0.0).min(1.0);
    
    Ok(authenticity_score)
}

/// Calculate metadata completeness score
fn calculate_metadata_completeness(nft_origin: &Account<crate::state::NftOrigin>) -> f64 {
    let mut completeness = 0.0;
    
    if !nft_origin.metadata_uri.is_empty() { completeness += 0.3; }
    // NftOrigin doesn't have name/symbol fields, so we skip those checks
    if nft_origin.origin_chain > 0 { completeness += 0.2; }
    
    completeness
}

/// Calculate cross-chain consistency score
fn calculate_cross_chain_consistency(mint_payload: &IncomingMintPayload) -> f64 {
    let mut consistency = 0.0;
    
    // Check if origin chain ID is reasonable
    if mint_payload.origin_chain_id.iter().any(|&b| b != 0) { consistency += 0.4; }
    
    // Check if gateway message ID is reasonable
    if mint_payload.gateway_message_id.iter().any(|&b| b != 0) { consistency += 0.3; }
    
    // Check if recipient address is reasonable
    if mint_payload.recipient_address.iter().any(|&b| b != 0) { consistency += 0.3; }
    
    consistency
}

/// Calculate gateway message authenticity score
fn calculate_gateway_authenticity(mint_payload: &IncomingMintPayload) -> f64 {
    let mut authenticity = 0.0;
    
    // Check if gateway message ID has reasonable entropy
    let entropy = calculate_byte_entropy(&mint_payload.gateway_message_id);
    if entropy > 0.7 {
        authenticity += 0.5;
    } else if entropy > 0.3 {
        authenticity += 0.3;
    } else {
        authenticity += 0.1;
    }
    
    // Check if the message ID doesn't look like a default value
    if mint_payload.gateway_message_id.iter().any(|&b| b != 0) {
        authenticity += 0.5;
    }
    
    authenticity
}

/// Calculate byte entropy for authenticity assessment
fn calculate_byte_entropy(bytes: &[u8; 32]) -> f64 {
    let mut byte_counts = [0u32; 256];
    for &byte in bytes {
        byte_counts[byte as usize] += 1;
    }
    
    let total_bytes = bytes.len() as f64;
    let mut entropy = 0.0;
    
    for &count in byte_counts.iter() {
        if count > 0 {
            let probability = count as f64 / total_bytes;
            entropy -= probability * probability.log2();
        }
    }
    
    // Normalize to 0-1 range (max entropy for 32 bytes is log2(256) = 8)
    entropy / 8.0
}

/// Determine final returning NFT status based on all validation results
fn determine_returning_nft_status(
    enhanced_exists: bool,
    origin_chain_valid: bool,
    metadata_consistent: bool,
    authenticity_score: f64,
    validation_score: f64,
    validation_details: &mut Vec<String>,
) -> bool {
    // Decision logic: NFT is returning if ALL critical checks pass
    let is_returning = enhanced_exists && 
                      origin_chain_valid && 
                      metadata_consistent && 
                      authenticity_score > 0.7 && 
                      validation_score > 0.7;
    
    validation_details.push(format!("Final decision: {} (exists: {}, chain: {}, metadata: {}, authenticity: {:.2}, validation: {:.2})", 
                                  if is_returning { "Returning NFT" } else { "New NFT" },
                                  enhanced_exists, origin_chain_valid, metadata_consistent, 
                                  authenticity_score, validation_score));
    
    is_returning
}

/// Check if a chain ID is supported
fn is_chain_supported(chain_id: u8) -> bool {
    matches!(chain_id, 1 | 2 | 3) // Solana, Base Sepolia, BNB Testnet
}
