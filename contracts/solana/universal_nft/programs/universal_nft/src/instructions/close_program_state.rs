use anchor_lang::prelude::*;

pub fn close_program_state_handler(ctx: Context<crate::CloseProgramState>) -> Result<()> {
    msg!("Program state account closed");
    Ok(())
}
