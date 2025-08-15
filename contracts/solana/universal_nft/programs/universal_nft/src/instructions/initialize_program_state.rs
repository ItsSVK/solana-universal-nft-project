use anchor_lang::prelude::*;

pub fn initialize_program_state_handler(ctx: Context<crate::InitializeProgramState>) -> Result<()> {
    let program_state = &mut ctx.accounts.program_state;
    program_state.bump = ctx.bumps.program_state;
    program_state.next_token_id = 1;
    program_state.collection_mint = Pubkey::default();
    program_state.collection_verified = false;
    Ok(())
}
