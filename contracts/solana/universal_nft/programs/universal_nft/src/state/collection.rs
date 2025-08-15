use anchor_lang::prelude::*;

#[account]
pub struct Collection {
    pub bump: u8,
    pub mint: Pubkey,
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub verified: bool,
    pub created_at: i64,
}

impl Collection {
    pub const LEN: usize = 8 + 1 + 32 + 32 + 10 + 200 + 1 + 8; // discriminator + fields
}
