use anchor_lang::prelude::*;

/// Outgoing Replay Protection account to prevent duplicate cross-chain message sending
/// 
/// This account stores information about sent cross-chain messages to prevent
/// duplicate transfers and ensure message uniqueness. Each outgoing message must
/// be recorded in this PDA before transmission.
/// 
/// Features:
/// - Unique PDA derivation based on message content
/// - Timestamp-based expiration
/// - Chain-specific protection
/// - Comprehensive metadata for monitoring
#[account]
pub struct OutgoingReplayProtection {
    /// Bump seed for the PDA
    pub bump: u8,
    /// Unix timestamp when the message was sent
    pub processed_at: i64,
    /// Chain ID where the message originated
    pub origin_chain_id: u8,
    /// Chain ID where the message is being sent
    pub destination_chain_id: u8,
    /// Token ID being transferred
    pub token_id: u64,
    /// Unique message ID for replay protection
    pub message_id: [u8; 32],
    /// Optional additional metadata for debugging and monitoring
    pub metadata: Option<OutgoingReplayProtectionMetadata>,
    /// Unix timestamp when this protection expires
    pub expires_at: i64,
}

/// Additional metadata for outgoing replay protection accounts
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct OutgoingReplayProtectionMetadata {
    /// The source chain identifier (e.g., "Solana", "Base Sepolia")
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

impl OutgoingReplayProtection {
    /// Calculate the space required for the OutgoingReplayProtection account
    /// 
    /// Breakdown:
    /// - 8 bytes: Account discriminator
    /// - 1 byte: bump seed
    /// - 8 bytes: processed_at timestamp (i64)
    /// - 1 byte: origin_chain_id (u8)
    /// - 1 byte: destination_chain_id (u8)
    /// - 8 bytes: token_id (u64)
    /// - 32 bytes: message_id
    /// - 1 byte: metadata option flag
    /// - Variable: metadata content (if present)
    /// - 8 bytes: expires_at timestamp (i64)
    /// Total: 67+ bytes (depending on metadata)
    pub const LEN: usize = 8 + 1 + 8 + 1 + 1 + 8 + 32 + 1 + 100 + 8; // 100 bytes for metadata
    
    /// Initialize a new OutgoingReplayProtection account
    pub fn initialize(
        &mut self,
        bump: u8,
        origin_chain_id: u8,
        destination_chain_id: u8,
        token_id: u64,
        message_id: [u8; 32],
        metadata: Option<OutgoingReplayProtectionMetadata>,
        expiry_seconds: i64,
    ) -> Result<()> {
        self.bump = bump;
        self.processed_at = Clock::get()?.unix_timestamp;
        self.origin_chain_id = origin_chain_id;
        self.destination_chain_id = destination_chain_id;
        self.token_id = token_id;
        self.message_id = message_id;
        self.metadata = metadata;
        self.expires_at = Clock::get()?.unix_timestamp + expiry_seconds;
        Ok(())
    }
    
    /// Check if this replay protection account matches the given parameters
    pub fn matches(&self, origin_chain_id: u8, destination_chain_id: u8, token_id: u64, message_id: &[u8; 32]) -> bool {
        self.origin_chain_id == origin_chain_id &&
        self.destination_chain_id == destination_chain_id &&
        self.token_id == token_id &&
        self.message_id == *message_id
    }
    
    /// Check if this protection has expired
    pub fn is_expired(&self) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        current_time > self.expires_at
    }
    
    /// Get the message ID as a string for logging
    pub fn get_message_id_string(&self) -> String {
        // Simple hex encoding without external crate
        self.message_id.iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>()
    }
    
    /// Get the chain names for logging
    pub fn get_chain_names(&self) -> (String, String) {
        let origin_name = match self.origin_chain_id {
            1 => "Solana".to_string(),
            2 => "Base Sepolia".to_string(),
            3 => "BNB Smart Chain Testnet".to_string(),
            _ => format!("Unknown Chain {}", self.origin_chain_id),
        };
        
        let destination_name = match self.destination_chain_id {
            1 => "Solana".to_string(),
            2 => "Base Sepolia".to_string(),
            3 => "BNB Smart Chain Testnet".to_string(),
            _ => format!("Unknown Chain {}", self.destination_chain_id),
        };
        
        (origin_name, destination_name)
    }
    
    /// Check if this message was sent recently (within a time window)
    pub fn is_recently_sent(&self, time_window_seconds: i64) -> bool {
        let current_time = Clock::get().unwrap().unix_timestamp;
        let time_diff = current_time - self.processed_at;
        time_diff <= time_window_seconds
    }
    
    /// Get the time remaining until expiration
    pub fn get_time_until_expiry(&self) -> i64 {
        let current_time = Clock::get().unwrap().unix_timestamp;
        self.expires_at - current_time
    }
    
    /// Check if this protection is still valid (not expired)
    pub fn is_valid(&self) -> bool {
        !self.is_expired()
    }
    
    /// Get a summary of the protection for logging
    pub fn get_summary(&self) -> String {
        let (origin_name, destination_name) = self.get_chain_names();
        format!(
            "OutgoingReplayProtection {{ token_id: {}, origin: {} -> {}, message_id: {}, expires_in: {}s }}",
            self.token_id,
            origin_name,
            destination_name,
            self.get_message_id_string(),
            self.get_time_until_expiry()
        )
    }
}
