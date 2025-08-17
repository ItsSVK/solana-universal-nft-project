use anchor_lang::prelude::*;
use crate::utils::validation_pipeline::*;
use crate::error::UniversalNftError;
use anchor_spl::associated_token::AssociatedToken;

#[derive(Accounts)]
pub struct MintNft<'info> {
    /// The account paying for the transaction
    pub payer: Signer<'info>,
    
    /// The mint account for the NFT
    pub mint: Account<'info, anchor_spl::token::Mint>,
    
    /// The token account that will hold the minted NFT
    pub token_account: Account<'info, anchor_spl::token::TokenAccount>,
    
    /// The program state account
    pub program_state: Account<'info, crate::state::ProgramState>,
    
    /// The replay protection account (PDA)
    pub replay_protection: Account<'info, crate::state::ReplayProtection>,
    
    /// The system program
    pub system_program: Program<'info, System>,
    
    /// The token program
    pub token_program: Program<'info, anchor_spl::token::Token>,
    
    /// The associated token program
    pub associated_token_program: Program<'info, AssociatedToken>,
}

/// Cross-chain NFT mint instruction parameters
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct CrossChainMintParams {
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
    /// Message hash for replay protection
    pub message_hash: Vec<u8>,
    /// Message timestamp
    pub timestamp: i64,

}

/// Mint NFT instruction handler with cross-chain validation
pub fn mint_nft_handler(ctx: Context<MintNft>, params: CrossChainMintParams) -> Result<()> {
    msg!("🚀 Starting cross-chain NFT mint instruction...");
    
    // Step 1: Validate cross-chain message using the validation pipeline
    let validation_context = CrossChainMessageContext {
        source_chain_id: params.source_chain_id,
        destination_chain_id: params.destination_chain_id,
        payload: params.payload.clone(),
        signatures: params.signatures.clone(),
        validators: params.validators.clone(),
        gateway_caller: params.gateway_caller,
        message_hash: params.message_hash.clone(),
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
    
    // Step 2: Check replay protection
    if ctx.accounts.replay_protection.processed_at > 0 {
        msg!("❌ Message already processed - replay protection triggered");
        return Err(UniversalNftError::MessageAlreadyProcessed.into());
    }
    
    // Step 3: Mark message as processed
    ctx.accounts.replay_protection.processed_at = Clock::get()?.unix_timestamp;
    
    // Convert chain_id to 32-byte array (pad with zeros)
    let mut chain_id_bytes = [0u8; 32];
    let source_chain_bytes = params.source_chain_id.to_le_bytes();
    chain_id_bytes[0] = source_chain_bytes[0];
    ctx.accounts.replay_protection.chain_id = chain_id_bytes;
    
    // Convert message_hash to 32-byte array (pad with zeros if needed)
    let mut message_hash_bytes = [0u8; 32];
    let message_len = std::cmp::min(params.message_hash.len(), 32);
    message_hash_bytes[..message_len].copy_from_slice(&params.message_hash[..message_len]);
    ctx.accounts.replay_protection.message_hash = message_hash_bytes;
    
    msg!("✅ Replay protection updated");
    
    // Step 4: Mint the NFT
    msg!("🎨 Minting NFT with metadata...");
    
    // For now, we'll use a simple mint without PDA signing
    // In a real implementation, this would use proper PDA derivation
    msg!("🎨 Minting NFT...");
    
    // Mint 1 token to the token account
    anchor_spl::token::mint_to(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            anchor_spl::token::MintTo {
                mint: ctx.accounts.mint.to_account_info(),
                to: ctx.accounts.token_account.to_account_info(),
                authority: ctx.accounts.program_state.to_account_info(),
            },
        ),
        1, // Mint 1 NFT
    )?;
    
    msg!("✅ NFT minted successfully!");
    
    msg!("🎉 Cross-chain NFT mint instruction completed successfully!");
    msg!("📊 Final Statistics:");
    msg!("   - Validation time: {}ms", validation_result.stats.total_validation_time_ms);
    msg!("   - Validation success rate: {}%", validation_result.stats.success_rate_percentage);
    msg!("   - NFT minted: {}", ctx.accounts.mint.key());
    msg!("   - Token account: {}", ctx.accounts.token_account.key());
    
    Ok(())
}
