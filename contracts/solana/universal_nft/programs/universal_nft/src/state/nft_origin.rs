use anchor_lang::prelude::*;

#[account]
pub struct NftOrigin {
    pub bump: u8,
    pub token_id: u64,
    pub origin_chain: u8,
    pub origin_address: [u8; 32],
    pub mint_address: Pubkey,
    pub created_at: i64,
    pub metadata_uri: String,
}

impl NftOrigin {
    pub const LEN: usize = 8 + 1 + 8 + 1 + 32 + 32 + 8 + 200; // discriminator + fields
}
