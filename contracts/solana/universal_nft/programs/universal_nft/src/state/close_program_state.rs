use anchor_lang::prelude::*;
use crate::state::ProgramState;

#[derive(Accounts)]
pub struct CloseProgramState<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(
        mut,
        close = payer,
    )]
    pub program_state: Account<'info, ProgramState>,
}
