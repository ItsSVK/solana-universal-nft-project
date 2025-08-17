use anchor_lang::prelude::*;
use crate::state::ReplayProtection;

#[derive(Accounts)]
pub struct CloseReplayProtection<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    /// The replay protection account to close
    #[account(
        mut,
        close = payer,
    )]
    pub replay_protection: Account<'info, ReplayProtection>,
}

pub fn close_replay_protection_handler(ctx: Context<CloseReplayProtection>) -> Result<()> {
    msg!("Gateway Message ID: {:?}", ctx.accounts.replay_protection.gateway_message_id);
    msg!("Closing replay protection account");
    Ok(())
}
