use anchor_lang::prelude::*;

#[account]
pub struct CrossChainMessage {
    pub bump: u8,
    pub message_id: [u8; 32],
    pub source_chain: u8,
    pub target_chain: u8,
    pub source_address: [u8; 32],
    pub target_address: [u8; 32],
    pub message_data: Vec<u8>,
    pub signature: [u8; 64],
    pub created_at: i64,
    pub processed: bool,
}

impl CrossChainMessage {
    pub const LEN: usize = 8 + 1 + 32 + 1 + 1 + 32 + 32 + 4 + 1024 + 64 + 8 + 1; // discriminator + fields
}
