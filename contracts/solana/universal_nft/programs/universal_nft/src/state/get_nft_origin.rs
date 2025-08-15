use anchor_lang::prelude::*;
use crate::state::NftOrigin;

#[derive(Accounts)]
#[instruction(token_id: u64)]
pub struct GetNftOrigin<'info> {
    /// The NFT origin account to fetch
    #[account(
        seeds = [
            b"nft_origin",
            &token_id.to_le_bytes(),
        ],
        bump,
        constraint = nft_origin.token_id == token_id @ crate::utils::nft_origin::ErrorCode::TokenIdMismatch,
    )]
    pub nft_origin: Account<'info, NftOrigin>,
}
