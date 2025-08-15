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
