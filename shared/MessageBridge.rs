use anchor_lang::prelude::*;
use solana_program::keccak;
use crate::CrossChainMessage::NFTTransferMessage;

/// Bridge utility for converting between different message formats
/// This module handles conversion between the shared CrossChainMessage format
/// and various chain-specific formats (Solana, EVM, etc.)

/// Convert shared NFTTransferMessage to Solana's internal format
pub fn to_solana_format(message: &NFTTransferMessage) -> SolanaNftMessage {
    SolanaNftMessage {
        message_type: 1, // NFT_MINT
        token_id: message.token_id,
        metadata_uri: message.metadata_uri.clone(),
        name: extract_name_from_uri(&message.metadata_uri), // Extract from metadata if available
        symbol: "UNFT".to_string(), // Default symbol
        origin_chain_id: message.origin_chain as u8,
        origin_address: message.origin_contract,
        recipient_address: message.recipient,
        timestamp: message.timestamp as i64,
        message_id: message.message_id,
        additional_metadata: None,
    }
}

/// Convert Solana format to shared NFTTransferMessage
pub fn from_solana_format(message: &SolanaNftMessage) -> Result<NFTTransferMessage> {
    Ok(NFTTransferMessage {
        token_id: message.token_id,
        metadata_uri: message.metadata_uri.clone(),
        recipient: message.recipient_address,
        origin_chain: message.origin_chain_id as u32,
        destination_chain: 0, // Will be set by caller
        message_id: message.message_id,
        timestamp: message.timestamp as u64,
        origin_contract: message.origin_address,
        nonce: 0, // Will be set by caller
    })
}

/// Convert shared NFTTransferMessage to EVM ABI format
pub fn to_evm_format(message: &NFTTransferMessage) -> Vec<u8> {
    // Using ethers-like ABI encoding
    let mut encoded = Vec::new();
    
    // Encode all fields according to Solidity ABI
    // tokenId (uint256 - 32 bytes)
    let mut token_id_bytes = [0u8; 32];
    token_id_bytes[24..].copy_from_slice(&message.token_id.to_be_bytes());
    encoded.extend_from_slice(&token_id_bytes);
    
    // Calculate dynamic offset for string data
    let base_offset = 32 * 9; // 9 fixed-size fields * 32 bytes each
    let mut dynamic_offset = base_offset;
    
    // metadataUri offset (uint256 - 32 bytes)
    let mut uri_offset_bytes = [0u8; 32];
    uri_offset_bytes[28..].copy_from_slice(&(dynamic_offset as u32).to_be_bytes());
    encoded.extend_from_slice(&uri_offset_bytes);
    
    // Update offset for next dynamic field
    let uri_length = message.metadata_uri.len();
    let uri_padded_length = ((uri_length + 31) / 32) * 32; // Round up to 32-byte boundary
    dynamic_offset += 32 + uri_padded_length; // 32 for length + padded data
    
    // recipient (bytes32)
    encoded.extend_from_slice(&message.recipient);
    
    // originChain (uint32 - padded to 32 bytes)
    let mut origin_chain_bytes = [0u8; 32];
    origin_chain_bytes[28..].copy_from_slice(&message.origin_chain.to_be_bytes());
    encoded.extend_from_slice(&origin_chain_bytes);
    
    // destinationChain (uint32 - padded to 32 bytes)
    let mut dest_chain_bytes = [0u8; 32];
    dest_chain_bytes[28..].copy_from_slice(&message.destination_chain.to_be_bytes());
    encoded.extend_from_slice(&dest_chain_bytes);
    
    // messageId (bytes32)
    encoded.extend_from_slice(&message.message_id);
    
    // timestamp (uint64 - padded to 32 bytes)
    let mut timestamp_bytes = [0u8; 32];
    timestamp_bytes[24..].copy_from_slice(&message.timestamp.to_be_bytes());
    encoded.extend_from_slice(&timestamp_bytes);
    
    // originContract (bytes32)
    encoded.extend_from_slice(&message.origin_contract);
    
    // nonce (uint256 - 32 bytes)
    let mut nonce_bytes = [0u8; 32];
    nonce_bytes[24..].copy_from_slice(&message.nonce.to_be_bytes());
    encoded.extend_from_slice(&nonce_bytes);
    
    // Now encode the dynamic data (metadataUri)
    // Length of string (uint256)
    let mut uri_length_bytes = [0u8; 32];
    uri_length_bytes[28..].copy_from_slice(&(uri_length as u32).to_be_bytes());
    encoded.extend_from_slice(&uri_length_bytes);
    
    // String data (padded to 32-byte boundary)
    let mut uri_data = message.metadata_uri.as_bytes().to_vec();
    while uri_data.len() % 32 != 0 {
        uri_data.push(0); // Pad with zeros
    }
    encoded.extend_from_slice(&uri_data);
    
    encoded
}

/// Convert EVM ABI format to shared NFTTransferMessage
pub fn from_evm_format(data: &[u8]) -> Result<NFTTransferMessage> {
    if data.len() < 32 * 9 {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    
    let mut offset = 0;
    
    // tokenId (uint256)
    let token_id = u64::from_be_bytes(data[offset + 24..offset + 32].try_into().unwrap());
    offset += 32;
    
    // metadataUri offset (uint256) - we'll read the actual string later
    let uri_offset = u32::from_be_bytes(data[offset + 28..offset + 32].try_into().unwrap()) as usize;
    offset += 32;
    
    // recipient (bytes32)
    let recipient = data[offset..offset + 32].try_into().unwrap();
    offset += 32;
    
    // originChain (uint32)
    let origin_chain = u32::from_be_bytes(data[offset + 28..offset + 32].try_into().unwrap());
    offset += 32;
    
    // destinationChain (uint32)
    let destination_chain = u32::from_be_bytes(data[offset + 28..offset + 32].try_into().unwrap());
    offset += 32;
    
    // messageId (bytes32)
    let message_id = data[offset..offset + 32].try_into().unwrap();
    offset += 32;
    
    // timestamp (uint64)
    let timestamp = u64::from_be_bytes(data[offset + 24..offset + 32].try_into().unwrap());
    offset += 32;
    
    // originContract (bytes32)
    let origin_contract = data[offset..offset + 32].try_into().unwrap();
    offset += 32;
    
    // nonce (uint256)
    let nonce = u64::from_be_bytes(data[offset + 24..offset + 32].try_into().unwrap());
    offset += 32;
    
    // Read metadataUri from dynamic data
    if uri_offset >= data.len() {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    
    let uri_length = u32::from_be_bytes(data[uri_offset + 28..uri_offset + 32].try_into().unwrap()) as usize;
    if uri_offset + 32 + uri_length > data.len() {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    
    let metadata_uri = String::from_utf8(
        data[uri_offset + 32..uri_offset + 32 + uri_length].to_vec()
    ).map_err(|_| ProgramError::InvalidInstructionData)?;
    
    Ok(NFTTransferMessage {
        token_id,
        metadata_uri,
        recipient,
        origin_chain,
        destination_chain,
        message_id,
        timestamp,
        origin_contract,
        nonce,
    })
}

/// Solana-specific message format (matches existing program structure)
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Debug, PartialEq)]
pub struct SolanaNftMessage {
    pub message_type: u8,
    pub token_id: u64,
    pub metadata_uri: String,
    pub name: String,
    pub symbol: String,
    pub origin_chain_id: u8,
    pub origin_address: [u8; 32],
    pub recipient_address: [u8; 32],
    pub timestamp: i64,
    pub message_id: [u8; 32],
    pub additional_metadata: Option<Vec<u8>>,
}

/// Validate cross-chain message format compatibility
pub fn validate_cross_chain_compatibility(message: &NFTTransferMessage) -> Result<()> {
    // Check token ID is valid
    if message.token_id == 0 {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    
    // Check metadata URI is not empty and not too long
    if message.metadata_uri.is_empty() || message.metadata_uri.len() > 500 {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    
    // Check timestamp is reasonable (not too old, not too far in future)
    let current_time = Clock::get()?.unix_timestamp as u64;
    let time_diff = if message.timestamp > current_time {
        message.timestamp - current_time
    } else {
        current_time - message.timestamp
    };
    
    if time_diff > 86400 { // 24 hours
        return Err(ProgramError::InvalidInstructionData.into());
    }
    
    // Check origin and destination chains are different
    if message.origin_chain == message.destination_chain {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    
    // Check message ID is not all zeros
    if message.message_id == [0u8; 32] {
        return Err(ProgramError::InvalidInstructionData.into());
    }
    
    Ok(())
}

/// Extract NFT name from metadata URI (simplified implementation)
fn extract_name_from_uri(uri: &str) -> String {
    // This is a simplified implementation
    // In production, you might want to fetch the metadata and extract the name
    format!("Universal NFT")
}

/// Generate message ID for cross-chain compatibility
pub fn generate_message_id(
    sender: &[u8; 32],
    token_id: u64,
    destination_chain: u32,
    nonce: u64,
    timestamp: u64,
) -> [u8; 32] {
    let mut data = Vec::new();
    data.extend_from_slice(sender);
    data.extend_from_slice(&token_id.to_le_bytes());
    data.extend_from_slice(&destination_chain.to_le_bytes());
    data.extend_from_slice(&nonce.to_le_bytes());
    data.extend_from_slice(&timestamp.to_le_bytes());
    
    keccak::hash(&data).to_bytes()
}

/// Chain ID constants for cross-chain compatibility
pub mod chain_ids {
    pub const SOLANA_DEVNET: u32 = 900;
    pub const ZETACHAIN_TESTNET: u32 = 7001;
    pub const BASE_SEPOLIA: u32 = 84532;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_evm_format_conversion() {
        let message = NFTTransferMessage {
            token_id: 123,
            metadata_uri: "https://example.com/metadata.json".to_string(),
            recipient: [1u8; 32],
            origin_chain: chain_ids::ZETACHAIN_TESTNET,
            destination_chain: chain_ids::BASE_SEPOLIA,
            message_id: [2u8; 32],
            timestamp: 1640995200,
            origin_contract: [3u8; 32],
            nonce: 1,
        };

        let encoded = to_evm_format(&message);
        let decoded = from_evm_format(&encoded).unwrap();

        assert_eq!(message.token_id, decoded.token_id);
        assert_eq!(message.metadata_uri, decoded.metadata_uri);
        assert_eq!(message.recipient, decoded.recipient);
        assert_eq!(message.origin_chain, decoded.origin_chain);
        assert_eq!(message.destination_chain, decoded.destination_chain);
        assert_eq!(message.message_id, decoded.message_id);
        assert_eq!(message.timestamp, decoded.timestamp);
        assert_eq!(message.origin_contract, decoded.origin_contract);
        assert_eq!(message.nonce, decoded.nonce);
    }

    #[test]
    fn test_message_id_generation() {
        let sender = [4u8; 32];
        let token_id = 123u64;
        let destination_chain = chain_ids::BASE_SEPOLIA;
        let nonce = 1u64;
        let timestamp = 1640995200u64;

        let message_id1 = generate_message_id(&sender, token_id, destination_chain, nonce, timestamp);
        let message_id2 = generate_message_id(&sender, token_id, destination_chain, nonce, timestamp);
        
        assert_eq!(message_id1, message_id2);
        
        // Different parameters should generate different IDs
        let message_id3 = generate_message_id(&sender, token_id, destination_chain, nonce + 1, timestamp);
        assert_ne!(message_id1, message_id3);
    }

    #[test]
    fn test_validation() {
        let valid_message = NFTTransferMessage {
            token_id: 123,
            metadata_uri: "https://example.com/metadata.json".to_string(),
            recipient: [1u8; 32],
            origin_chain: chain_ids::ZETACHAIN_TESTNET,
            destination_chain: chain_ids::BASE_SEPOLIA,
            message_id: [2u8; 32],
            timestamp: Clock::get().unwrap().unix_timestamp as u64,
            origin_contract: [3u8; 32],
            nonce: 1,
        };

        assert!(validate_cross_chain_compatibility(&valid_message).is_ok());

        // Test invalid token ID
        let mut invalid_message = valid_message.clone();
        invalid_message.token_id = 0;
        assert!(validate_cross_chain_compatibility(&invalid_message).is_err());

        // Test same chain transfer
        let mut invalid_message = valid_message.clone();
        invalid_message.destination_chain = invalid_message.origin_chain;
        assert!(validate_cross_chain_compatibility(&invalid_message).is_err());
    }
}