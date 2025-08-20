use anchor_lang::prelude::*;
use solana_program::keccak;

/// Shared message format for Universal NFT Protocol cross-chain transfers
/// This struct must be serializable/deserializable across Solana (Rust) and EVM (Solidity)
#[derive(AnchorSerialize, AnchorDeserialize, Debug, Clone, PartialEq)]
pub struct NFTTransferMessage {
    pub token_id: u64,           // NFT token ID (using u64 for Solana compatibility)
    pub metadata_uri: String,    // IPFS/HTTP URL to NFT metadata
    pub recipient: [u8; 32],     // Recipient address (32 bytes to support both EVM and Solana)
    pub origin_chain: u32,       // Chain ID where NFT was originally minted
    pub destination_chain: u32,  // Target chain ID
    pub message_id: [u8; 32],    // Unique message identifier for replay protection
    pub timestamp: u64,          // Block timestamp when message was created
    pub origin_contract: [u8; 32], // Original contract address where NFT was first minted
    pub nonce: u64,             // Sender's nonce for additional uniqueness
}

impl NFTTransferMessage {
    /// Create a new NFT transfer message
    pub fn new(
        token_id: u64,
        metadata_uri: String,
        recipient: [u8; 32],
        origin_chain: u32,
        destination_chain: u32,
        origin_contract: [u8; 32],
        sender: Pubkey,
        nonce: u64,
        clock: &Clock,
    ) -> Result<Self> {
        let message_id = Self::generate_message_id(sender, token_id, destination_chain, nonce, clock.unix_timestamp as u64);
        
        Ok(Self {
            token_id,
            metadata_uri,
            recipient,
            origin_chain,
            destination_chain,
            message_id,
            timestamp: clock.unix_timestamp as u64,
            origin_contract,
            nonce,
        })
    }

    /// Generate unique message ID using keccak hash
    pub fn generate_message_id(
        sender: Pubkey,
        token_id: u64,
        destination_chain: u32,
        nonce: u64,
        timestamp: u64,
    ) -> [u8; 32] {
        let mut data = Vec::new();
        data.extend_from_slice(&sender.to_bytes());
        data.extend_from_slice(&token_id.to_le_bytes());
        data.extend_from_slice(&destination_chain.to_le_bytes());
        data.extend_from_slice(&nonce.to_le_bytes());
        data.extend_from_slice(&timestamp.to_le_bytes());
        
        keccak::hash(&data).to_bytes()
    }

    /// Validate message format and constraints
    pub fn validate(&self, max_age: u64, current_timestamp: u64) -> Result<()> {
        require!(self.timestamp > 0, UniversalNftError::InvalidTimestamp);
        require!(
            current_timestamp - self.timestamp <= max_age,
            UniversalNftError::MessageTooOld
        );
        require!(!self.metadata_uri.is_empty(), UniversalNftError::EmptyMetadataUri);
        require!(
            self.metadata_uri.len() <= 500,
            UniversalNftError::MetadataUriTooLong
        );
        require!(
            self.origin_chain != self.destination_chain,
            UniversalNftError::SameChainTransfer
        );
        
        Ok(())
    }

    /// Serialize message for cross-chain transmission (compatible with Solidity abi.encode)
    pub fn encode(&self) -> Result<Vec<u8>> {
        // Use borsh serialization for Solana compatibility
        // The Gateway will handle conversion to EVM-compatible format
        borsh::to_vec(self).map_err(|_| UniversalNftError::SerializationFailed.into())
    }

    /// Deserialize message from cross-chain transmission
    pub fn decode(data: &[u8]) -> Result<Self> {
        borsh::from_slice(data).map_err(|_| UniversalNftError::DeserializationFailed.into())
    }

    /// Convert Ethereum address (20 bytes) to 32-byte format
    pub fn ethereum_address_to_bytes32(eth_address: &[u8; 20]) -> [u8; 32] {
        let mut result = [0u8; 32];
        result[12..].copy_from_slice(eth_address);
        result
    }

    /// Convert 32-byte format to Ethereum address (20 bytes)
    pub fn bytes32_to_ethereum_address(bytes32: &[u8; 32]) -> [u8; 20] {
        let mut result = [0u8; 20];
        result.copy_from_slice(&bytes32[12..]);
        result
    }

    /// Convert Solana pubkey to 32-byte format
    pub fn solana_pubkey_to_bytes32(pubkey: &Pubkey) -> [u8; 32] {
        pubkey.to_bytes()
    }

    /// Convert 32-byte format to Solana pubkey
    pub fn bytes32_to_solana_pubkey(bytes32: &[u8; 32]) -> Result<Pubkey> {
        Pubkey::try_from(bytes32.as_slice()).map_err(|_| UniversalNftError::InvalidPubkey.into())
    }
}

/// Custom error codes for Universal NFT operations
#[error_code]
pub enum UniversalNftError {
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    #[msg("Message too old")]
    MessageTooOld,
    #[msg("Empty metadata URI")]
    EmptyMetadataUri,
    #[msg("Metadata URI too long")]
    MetadataUriTooLong,
    #[msg("Same chain transfer")]
    SameChainTransfer,
    #[msg("Serialization failed")]
    SerializationFailed,
    #[msg("Deserialization failed")]
    DeserializationFailed,
    #[msg("Invalid pubkey")]
    InvalidPubkey,
}

/// Chain IDs for supported networks
pub mod chain_ids {
    pub const SOLANA_DEVNET: u32 = 900; // Custom ID for Solana devnet
    pub const ZETACHAIN_TESTNET: u32 = 7001;
    pub const BASE_SEPOLIA: u32 = 84532;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_id_generation() {
        let sender = Pubkey::new_unique();
        let token_id = 123u64;
        let destination_chain = chain_ids::BASE_SEPOLIA;
        let nonce = 1u64;
        let timestamp = 1640995200u64;

        let message_id = NFTTransferMessage::generate_message_id(
            sender, token_id, destination_chain, nonce, timestamp
        );

        // Should generate consistent hash
        let message_id2 = NFTTransferMessage::generate_message_id(
            sender, token_id, destination_chain, nonce, timestamp
        );
        assert_eq!(message_id, message_id2);
    }

    #[test]
    fn test_address_conversions() {
        let eth_addr: [u8; 20] = [1; 20];
        let bytes32 = NFTTransferMessage::ethereum_address_to_bytes32(&eth_addr);
        let converted_back = NFTTransferMessage::bytes32_to_ethereum_address(&bytes32);
        assert_eq!(eth_addr, converted_back);

        let solana_pubkey = Pubkey::new_unique();
        let bytes32 = NFTTransferMessage::solana_pubkey_to_bytes32(&solana_pubkey);
        let converted_back = NFTTransferMessage::bytes32_to_solana_pubkey(&bytes32).unwrap();
        assert_eq!(solana_pubkey, converted_back);
    }
}