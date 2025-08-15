use anchor_lang::prelude::*;
use crate::constants::*;
use crate::state::ProgramState;

#[derive(Accounts)]
pub struct InitializeProgramState<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(
        init,
        payer = payer,
        seeds = [PROGRAM_STATE_SEED],
        bump,
        space = ProgramState::LEN,
    )]
    pub program_state: Account<'info, ProgramState>,
    
    pub system_program: Program<'info, System>,
}

// impl Collection {
//     pub const LEN: usize = 8 + 1 + 32 + 32 + 10 + 200 + 1 + 8; // discriminator + fields
// }
