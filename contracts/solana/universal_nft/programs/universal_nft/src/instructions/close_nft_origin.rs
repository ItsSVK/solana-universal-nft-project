use anchor_lang::prelude::*;

pub fn close_nft_origin_handler(ctx: Context<crate::CloseNftOrigin>) -> Result<()> {
    msg!("NFT Origin account closed");
    Ok(())
}
