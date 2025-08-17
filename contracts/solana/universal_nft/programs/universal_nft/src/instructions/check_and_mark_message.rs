use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(chain_id: [u8; 32], gateway_message_id: [u8; 32])]
pub struct CheckAndMarkMessage<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// The replay protection PDA to check and mark
    #[account(
        init_if_needed,
        payer = payer,
        space = ReplayProtection::LEN,
        seeds = [REPLAY_SEED, &chain_id, &gateway_message_id],
        bump
    )]
    pub replay_protection: Account<'info, ReplayProtection>,
    
    /// The system program for account creation
    pub system_program: Program<'info, System>,
}

/// Check if a Gateway message has been processed and mark it as processed if not
/// 
/// This function implements atomic check-and-mark logic for replay protection
/// using ZetaChain Gateway message IDs. It will:
/// 1. Check if the replay protection PDA already exists for the Gateway message ID
/// 2. If it doesn't exist, create it and mark the message as processed
/// 3. If it already exists, return an error indicating replay detected
/// 
/// This provides enhanced security by using official Gateway message identifiers
/// instead of custom message hashes.
pub fn check_and_mark_message_handler(
    ctx: Context<CheckAndMarkMessage>,
    chain_id: [u8; 32],
    gateway_message_id: [u8; 32],
) -> Result<()> {
    let replay_protection = &mut ctx.accounts.replay_protection;
    
    msg!("🔍 Checking Gateway replay protection...");
    msg!("Chain ID: {}", replay_protection.get_chain_id_string());
    msg!("Gateway Message ID: {}", replay_protection.get_gateway_message_id_string());
    
    // Check if this account was just created (init_if_needed)
    if replay_protection.processed_at == 0 {
        // This is a new Gateway message, mark it as processed
        replay_protection.initialize(
            0, // Placeholder bump - in production this should be properly derived
            chain_id,
            gateway_message_id,
            None, // No metadata for now - can be enhanced later
        )?;
        
        msg!("✅ Gateway message marked as processed!");
        msg!("   Chain ID: {}", replay_protection.get_chain_id_string());
        msg!("   Gateway Message ID: {}", replay_protection.get_gateway_message_id_string());
        msg!("   Processed at: {}", replay_protection.processed_at);
        msg!("   Processing age: {} seconds", replay_protection.get_processing_age_seconds());
    } else {
        // This Gateway message has already been processed - replay detected!
        msg!("🚨 REPLAY DETECTED - Gateway message already processed!");
        msg!("   Chain ID: {}", replay_protection.get_chain_id_string());
        msg!("   Gateway Message ID: {}", replay_protection.get_gateway_message_id_string());
        msg!("   Originally processed at: {}", replay_protection.processed_at);
        msg!("   Processing age: {} seconds", replay_protection.get_processing_age_seconds());
        
        // Check if this is a recent replay attempt
        if replay_protection.is_recently_processed(3600) { // Within last hour
            msg!("⚠️  Recent replay attempt detected (within last hour)");
        }
        
        return err!(UniversalNftError::MessageAlreadyProcessed);
    }
    
    Ok(())
}

/// Enhanced check and mark with metadata
/// 
/// This version allows passing additional metadata for better tracking
/// and debugging of cross-chain messages.
#[derive(Accounts)]
#[instruction(
    chain_id: [u8; 32], 
    gateway_message_id: [u8; 32],
    metadata: Option<ReplayProtectionMetadata>
)]
pub struct CheckAndMarkMessageWithMetadata<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// The replay protection PDA to check and mark
    #[account(
        init_if_needed,
        payer = payer,
        space = ReplayProtection::LEN,
        seeds = [REPLAY_SEED, &chain_id, &gateway_message_id],
        bump
    )]
    pub replay_protection: Account<'info, ReplayProtection>,
    
    /// The system program for account creation
    pub system_program: Program<'info, System>,
}

/// Check and mark Gateway message with optional metadata
pub fn check_and_mark_message_with_metadata_handler(
    ctx: Context<CheckAndMarkMessageWithMetadata>,
    chain_id: [u8; 32],
    gateway_message_id: [u8; 32],
    metadata: Option<ReplayProtectionMetadata>,
) -> Result<()> {
    let replay_protection = &mut ctx.accounts.replay_protection;
    
    msg!("🔍 Checking Gateway replay protection with metadata...");
    msg!("Chain ID: {}", replay_protection.get_chain_id_string());
    msg!("Gateway Message ID: {}", replay_protection.get_gateway_message_id_string());
    
    if let Some(ref meta) = metadata {
        msg!("📋 Metadata provided:");
        msg!("   Source Chain: {}", meta.source_chain_name);
        msg!("   Destination Chain: {}", meta.destination_chain_name);
        msg!("   Token ID: {}", meta.token_id);
        msg!("   Recipient: {}", meta.recipient_address.iter().map(|b| format!("{:02x}", b)).collect::<String>());
        msg!("   Context: {}", meta.context);
    }
    
    // Check if this account was just created (init_if_needed)
    if replay_protection.processed_at == 0 {
        // This is a new Gateway message, mark it as processed with metadata
        replay_protection.initialize(
            0, // Placeholder bump - in production this should be properly derived
            chain_id,
            gateway_message_id,
            metadata,
        )?;
        
        msg!("✅ Gateway message marked as processed with metadata!");
        msg!("   Chain ID: {}", replay_protection.get_chain_id_string());
        msg!("   Gateway Message ID: {}", replay_protection.get_gateway_message_id_string());
        msg!("   Processed at: {}", replay_protection.processed_at);
        msg!("   Has metadata: {}", replay_protection.metadata.is_some());
    } else {
        // This Gateway message has already been processed - replay detected!
        msg!("🚨 REPLAY DETECTED - Gateway message already processed!");
        msg!("   Chain ID: {}", replay_protection.get_chain_id_string());
        msg!("   Gateway Message ID: {}", replay_protection.get_gateway_message_id_string());
        msg!("   Originally processed at: {}", replay_protection.processed_at);
        
        return err!(UniversalNftError::MessageAlreadyProcessed);
    }
    
    Ok(())
}
