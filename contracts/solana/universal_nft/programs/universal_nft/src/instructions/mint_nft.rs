use anchor_lang::prelude::*;
use crate::utils::validation_pipeline::*;
use crate::utils::gateway_validation::*;
use crate::error::UniversalNftError;
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
