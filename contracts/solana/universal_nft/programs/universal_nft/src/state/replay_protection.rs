use anchor_lang::prelude::*;

/// Replay Protection account to prevent duplicate cross-chain message processing
/// 
/// This account stores information about processed cross-chain messages to prevent
/// replay attacks. Each incoming message must be checked against this PDA before
/// any state changes occur.
#[account]
pub struct ReplayProtection {
    /// Bump seed for the PDA
    pub bump: u8,
    /// Unix timestamp when the message was processed
    pub processed_at: i64,
    /// Chain ID where the message originated (32 bytes for future extensibility)
    pub chain_id: [u8; 32],
    /// Hash of the processed message for uniqueness verification
    pub message_hash: [u8; 32],
}

impl ReplayProtection {
    /// Calculate the space required for the ReplayProtection account
    /// 
    /// Breakdown:
    /// - 8 bytes: Account discriminator
    /// - 1 byte: bump seed
    /// - 8 bytes: processed_at timestamp (i64)
    /// - 32 bytes: chain_id
    /// - 32 bytes: message_hash
    /// Total: 81 bytes
    pub const LEN: usize = 8 + 1 + 8 + 32 + 32;
    
    /// Initialize a new ReplayProtection account
    pub fn initialize(&mut self, bump: u8, chain_id: [u8; 32], message_hash: [u8; 32]) -> Result<()> {
        self.bump = bump;
        self.processed_at = Clock::get()?.unix_timestamp;
        self.chain_id = chain_id;
        self.message_hash = message_hash;
        Ok(())
    }
    
    /// Check if this replay protection account matches the given parameters
    pub fn matches(&self, chain_id: &[u8; 32], message_hash: &[u8; 32]) -> bool {
        self.chain_id == *chain_id && self.message_hash == *message_hash
    }
}
