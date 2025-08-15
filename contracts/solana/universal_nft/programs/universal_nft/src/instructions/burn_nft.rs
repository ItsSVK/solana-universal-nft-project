use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct BurnNft<'info> {
    // TODO: Define accounts for burn_nft
    pub owner: Signer<'info>,
}

// TODO: Implement burn_nft instruction
pub fn burn_nft_handler(_ctx: Context<BurnNft>) -> Result<()> {
    msg!("burn_nft instruction not yet implemented");
    Ok(())
}
