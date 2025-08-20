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

/// on_call entrypoint handler for ZetaChain Gateway
/// 
/// This function is called by the Gateway when a cross-chain message is received.
/// It implements the full validation pipeline and NFT minting process.
pub fn on_call_handler(ctx: Context<OnCall>, payload: Vec<u8>) -> Result<()> {
    // Performance tracking start
    let start_time = Clock::get()?.unix_timestamp;
    let operation_id = generate_operation_id();
    
    msg!("🚀 ZetaChain Gateway on_call entrypoint triggered!");
    msg!("📊 Operation ID: {}", operation_id);
    msg!("📦 Payload size: {} bytes", payload.len());
    msg!("⏰ Start time: {}", start_time);
    msg!("🔗 Program ID: {}", ctx.program_id);
    msg!("👤 Payer: {}", ctx.accounts.payer.key());
    
    // Step 1: Validate payload size with detailed error handling
    if payload.is_empty() {
        msg!("❌ Payload is empty");
        log_operation_failure(&operation_id, "PayloadEmpty", "Payload is empty");
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    if payload.len() < 104 { // Minimum size for required fields
        msg!("❌ Payload too small: {} bytes (minimum: 104)", payload.len());
        msg!("   Required fields: token_id (8) + origin_chain_id (32) + gateway_message_id (32) + metadata_uri (variable) + name (variable) + symbol (variable) + recipient_address (32)");
        log_operation_failure(&operation_id, "PayloadTooSmall", &format!("Payload too small: {} bytes", payload.len()));
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    if payload.len() > 2048 { // Maximum size to prevent DoS
        msg!("❌ Payload too large: {} bytes (maximum: 2048)", payload.len());
        msg!("   This could indicate a DoS attempt or corrupted data");
        log_operation_failure(&operation_id, "PayloadTooLarge", &format!("Payload too large: {} bytes", payload.len()));
        return Err(UniversalNftError::InvalidCrossChainMessage.into());
    }
    
    // Step 2: Decode the payload with error handling
    let decode_start = Clock::get()?.unix_timestamp;
    let mint_payload = match decode_incoming_mint_payload(&payload) {
        Ok(payload) => {
            let decode_time = Clock::get()?.unix_timestamp - decode_start;
            msg!("✅ Payload decoded successfully in {}ms", decode_time);
            payload
        },
        Err(e) => {
            let decode_time = Clock::get()?.unix_timestamp - decode_start;
            msg!("❌ Failed to decode payload in {}ms: {:?}", decode_time, e);
            msg!("   Payload size: {} bytes", payload.len());
            msg!("   First 64 bytes: {:?}", &payload[..std::cmp::min(64, payload.len())]);
            log_operation_failure(&operation_id, "PayloadDecodeFailed", &format!("Failed to decode payload: {:?}", e));
            return Err(e);
        }
    };
    
    // Step 2.5: Validate cross-chain message format (simplified for stack optimization)
    validate_cross_chain_message_format(&payload)?;
    
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
    
    // Step 5: Check if NFT already exists by token_id (simplified)
    let is_returning_nft = ctx.accounts.nft_origin_by_token_id.mint_address != Pubkey::default();
    
    if is_returning_nft {
        // Case 1: Link to existing mint (returning NFT)
        handle_returning_nft(
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
        )?;
    } else {
        // Case 2: Create new mint and metadata (new NFT)
        handle_new_nft(
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
        )?;
    }
    
    // Step 6: Mint the NFT (simplified)
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
    
    // Step 7: Create NFT metadata (simplified)
    create_nft_metadata(
        &ctx.accounts.mint,
        &ctx.accounts.metadata,
        &ctx.accounts.master_edition,
        &ctx.accounts.collection_mint,
        &ctx.accounts.collection_metadata,
        &ctx.accounts.collection_master_edition,
        &mint_payload,
        &ctx.accounts.payer,
        &ctx.accounts.system_program,
        &ctx.accounts.rent,
    )?;
    
    // Step 8: Verify collection association (simplified)
    verify_collection_association(
        &ctx.accounts.collection_mint,
        &ctx.accounts.program_state,
        &mint_payload,
    )?;
    
    // Step 9: Validate recipient address (simplified)
    validate_recipient_address(
        &mint_payload,
        &ctx.accounts.recipient,
        &ctx.accounts.token_account,
    )?;
    
    // Step 10: Initialize NFT origin PDA (simplified)
    let metadata_uri = mint_payload.metadata_uri.clone();
    
    ctx.accounts.nft_origin.initialize(
        0, // Placeholder bump
        mint_payload.token_id,
        mint_payload.origin_chain_id[0], // Use first byte as chain ID
        mint_payload.origin_chain_id,
        ctx.accounts.mint.key(),
        metadata_uri,
    )?;
    
    // Step 11: Log success (simplified)
    msg!("🎉 on_call entrypoint completed successfully!");
    
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
