use anchor_lang::prelude::*;
use crate::state::NftOrigin;

#[derive(Accounts)]
pub struct CloseNftOrigin<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(
        mut,
        close = payer,
    )]
    pub nft_origin: Account<'info, NftOrigin>,
}
