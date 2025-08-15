use anchor_lang::prelude::*;

#[account]
pub struct ReplayProtection {
    pub bump: u8,
    pub message_hash: [u8; 32],
    pub processed_at: i64,
    pub chain_id: u8,
    pub message_id: [u8; 32],
}

impl ReplayProtection {
    pub const LEN: usize = 8 + 1 + 32 + 8 + 1 + 32; // discriminator + fields
}
