use anchor_lang::prelude::*;
use crate::state;

#[derive(Accounts)]
pub struct InitializeProgramState<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    #[account(
        init,
        payer = payer,
        seeds = [crate::constants::PROGRAM_STATE_SEED],
        bump,
        space = state::ProgramState::LEN,
    )]
    pub program_state: Account<'info, state::ProgramState>,
    pub system_program: Program<'info, System>,
}

// impl Collection {
//     pub const LEN: usize = 8 + 1 + 32 + 32 + 10 + 200 + 1 + 8; // discriminator + fields
// }
