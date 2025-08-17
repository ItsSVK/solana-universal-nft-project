use anchor_lang::prelude::*;
use crate::state::*;

/// Close a replay protection account and return rent to the payer
pub fn close_replay_protection_handler(ctx: Context<CloseReplayProtection>) -> Result<()> {
    let replay_protection = &ctx.accounts.replay_protection;
    
    msg!("Closing replay protection account for:");
    msg!("Chain ID: {:?}", replay_protection.chain_id);
    msg!("Gateway Message ID: {:?}", replay_protection.gateway_message_id);
    msg!("Processed at: {}", replay_protection.processed_at);
    
    // The account will be closed and rent returned to payer
    // This is handled by the `close = payer` constraint in the accounts struct
    
    Ok(())
}
