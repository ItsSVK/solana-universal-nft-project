use anchor_lang::prelude::*;
use crate::constants::*;
use crate::state::*;

#[derive(Accounts)]
#[instruction(token_id: u64)]
pub struct CreateNftOrigin<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(
        init,
        payer = payer,
        seeds = [NFT_ORIGIN_SEED, &token_id.to_le_bytes()],
        bump,
        space = NftOrigin::LEN,
    )]
    pub nft_origin: Account<'info, NftOrigin>,
    
    #[account(mut)]
    pub program_state: Account<'info, ProgramState>,
    
    pub system_program: Program<'info, System>,
}

// Helper struct to pass token_id as instruction data
pub struct CreateNftOriginData {
    pub token_id: u64,
    pub origin_chain: u8,
    pub origin_address: [u8; 32],
    pub mint_address: Pubkey,
    pub metadata_uri: String,
}
