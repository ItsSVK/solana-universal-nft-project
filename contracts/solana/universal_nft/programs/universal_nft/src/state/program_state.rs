use anchor_lang::prelude::*;

#[account]
pub struct ProgramState {
    pub bump: u8,
    pub next_token_id: u64,
    pub collection_mint: Pubkey,
    pub collection_verified: bool,
}

impl ProgramState {
    pub const LEN: usize = 8 + 1 + 8 + 32 + 1; // discriminator + fields
}

pub fn get_next_token_id(program_state: &mut Account<ProgramState>) -> u64 {
    let next = program_state.next_token_id;
    program_state.next_token_id = program_state
        .next_token_id
        .checked_add(1)
        .expect("token id overflow");
    next
}
