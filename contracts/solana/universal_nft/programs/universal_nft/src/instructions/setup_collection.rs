use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct SetupCollection<'info> {
    // TODO: Define accounts for setup_collection
    pub payer: Signer<'info>,
}

// TODO: Implement setup_collection instruction
pub fn setup_collection_handler(_ctx: Context<SetupCollection>) -> Result<()> {
    msg!("setup_collection instruction not yet implemented");
    Ok(())
}
