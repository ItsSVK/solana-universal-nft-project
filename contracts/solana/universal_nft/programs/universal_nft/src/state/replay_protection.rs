use anchor_lang::prelude::*;

/// Replay Protection account to prevent duplicate cross-chain message processing
/// 
/// This account stores information about processed cross-chain messages to prevent
/// replay attacks. Each incoming message must be checked against this PDA before
/// any state changes occur.
/// 
/// Updated to use ZetaChain Gateway message IDs for enhanced security and
/// compatibility with the Gateway system.
#[account]
pub struct ReplayProtection {
    /// Bump seed for the PDA
    pub bump: u8,
    /// Unix timestamp when the message was processed
    pub processed_at: i64,
    /// Chain ID where the message originated (32 bytes for future extensibility)
    pub chain_id: [u8; 32],
    /// ZetaChain Gateway message ID for uniqueness verification
    /// This replaces the custom message_hash with the official Gateway identifier
    pub gateway_message_id: [u8; 32],
    /// Optional additional metadata for debugging and monitoring
    pub metadata: Option<ReplayProtectionMetadata>,
}

/// Additional metadata for replay protection accounts
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct ReplayProtectionMetadata {
    /// The source chain identifier (e.g., "ethereum", "polygon")
    pub source_chain_name: String,
    /// The destination chain identifier
    pub destination_chain_name: String,
    /// The token ID being transferred
    pub token_id: u64,
    /// The recipient address on the destination chain
    pub recipient_address: [u8; 32],
    /// Additional context about the message
    pub context: String,
}

impl ReplayProtection {
    /// Calculate the space required for the ReplayProtection account
    /// 
    /// Breakdown:
    /// - 8 bytes: Account discriminator
    /// - 1 byte: bump seed
    /// - 8 bytes: processed_at timestamp (i64)
    /// - 32 bytes: chain_id
    /// - 32 bytes: gateway_message_id
    /// - 1 byte: metadata option flag
    /// - Variable: metadata content (if present)
    /// Total: 82+ bytes (depending on metadata)
    pub const LEN: usize = 8 + 1 + 8 + 32 + 32 + 1 + 100; // 100 bytes for metadata
    
    /// Initialize a new ReplayProtection account with Gateway message ID
    pub fn initialize(
        &mut self, 
        bump: u8, 
        chain_id: [u8; 32], 
        gateway_message_id: [u8; 32],
        metadata: Option<ReplayProtectionMetadata>,
    ) -> Result<()> {
        self.bump = bump;
        self.processed_at = Clock::get()?.unix_timestamp;
        self.chain_id = chain_id;
        self.gateway_message_id = gateway_message_id;
        self.metadata = metadata;
        Ok(())
    }
    
    /// Check if this replay protection account matches the given Gateway message ID
    pub fn matches_gateway_message(&self, chain_id: &[u8; 32], gateway_message_id: &[u8; 32]) -> bool {
        self.chain_id == *chain_id && self.gateway_message_id == *gateway_message_id
    }
    
    /// Check if this replay protection account matches the given parameters (legacy support)
    pub fn matches(&self, chain_id: &[u8; 32], message_hash: &[u8; 32]) -> bool {
        self.chain_id == *chain_id && self.gateway_message_id == *message_hash
    }
    
    /// Get the Gateway message ID as a string for logging
    pub fn get_gateway_message_id_string(&self) -> String {
        // Simple hex encoding without external crate
        self.gateway_message_id.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }
    
    /// Get the chain ID as a string for logging
    pub fn get_chain_id_string(&self) -> String {
        // Simple hex encoding without external crate
        self.chain_id.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }
    
    /// Check if this message was processed recently (within a time window)
    pub fn is_recently_processed(&self, time_window_seconds: i64) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        let time_diff = current_time - self.processed_at;
        time_diff < time_window_seconds
    }
    
    /// Get processing age in seconds
    pub fn get_processing_age_seconds(&self) -> i64 {
        let current_time = Clock::get().unwrap().unix_timestamp;
        current_time - self.processed_at
    }
}

/// Replay protection statistics for monitoring and debugging
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct ReplayProtectionStats {
    /// Total number of messages processed
    pub total_messages_processed: u64,
    /// Number of replay attempts detected
    pub replay_attempts_detected: u64,
    /// Number of messages processed in the last hour
    pub messages_last_hour: u64,
    /// Number of messages processed in the last 24 hours
    pub messages_last_24h: u64,
    /// Timestamp of last update
    pub last_updated: i64,
}

impl ReplayProtectionStats {
    /// Create new stats with default values
    pub fn new() -> Self {
        Self {
            total_messages_processed: 0,
            replay_attempts_detected: 0,
            messages_last_hour: 0,
            messages_last_24h: 0,
            last_updated: Clock::get().unwrap().unix_timestamp,
        }
    }
    
    /// Increment message processed count
    pub fn increment_messages_processed(&mut self) {
        self.total_messages_processed += 1;
        self.messages_last_hour += 1;
        self.messages_last_24h += 1;
        self.last_updated = Clock::get().unwrap().unix_timestamp;
    }
    
    /// Increment replay attempt count
    pub fn increment_replay_attempts(&mut self) {
        self.replay_attempts_detected += 1;
        self.last_updated = Clock::get().unwrap().unix_timestamp;
    }
    
    /// Reset hourly and daily counters (should be called periodically)
    pub fn reset_periodic_counters(&mut self) {
        self.messages_last_hour = 0;
        self.messages_last_24h = 0;
        self.last_updated = Clock::get().unwrap().unix_timestamp;
    }
}
