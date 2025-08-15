use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::*;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(chain_id: [u8; 32], message_hash: [u8; 32])]
pub struct CheckAndMarkMessage<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// The replay protection PDA to check and mark
    #[account(
        init_if_needed,
        payer = payer,
        space = ReplayProtection::LEN,
        seeds = [REPLAY_SEED, &chain_id, &message_hash],
        bump,
    )]
    pub replay_protection: Account<'info, ReplayProtection>,
    
    pub system_program: Program<'info, System>,
}

/// Check if a message has been processed and mark it as processed if not
/// 
/// This function implements atomic check-and-mark logic for replay protection.
/// It will:
/// 1. Check if the replay protection PDA already exists
/// 2. If it doesn't exist, create it and mark the message as processed
/// 3. If it already exists, return an error indicating replay detected
pub fn check_and_mark_message_handler(
    ctx: Context<CheckAndMarkMessage>,
    chain_id: [u8; 32],
    message_hash: [u8; 32],
) -> Result<()> {
    let replay_protection = &mut ctx.accounts.replay_protection;
    
    // Check if this account was just created (init_if_needed)
    if replay_protection.processed_at == 0 {
        // This is a new message, mark it as processed
        replay_protection.initialize(
            ctx.bumps.replay_protection,
            chain_id,
            message_hash,
        )?;
        
        msg!("Message marked as processed - Chain ID: {:?}, Hash: {:?}", chain_id, message_hash);
        msg!("Processed at: {}", replay_protection.processed_at);
    } else {
        // This message has already been processed - replay detected!
        msg!("REPLAY DETECTED - Message already processed at: {}", replay_protection.processed_at);
        msg!("Chain ID: {:?}, Hash: {:?}", chain_id, message_hash);
        
        return err!(UniversalNftError::MessageAlreadyProcessed);
    }
    
    Ok(())
}
