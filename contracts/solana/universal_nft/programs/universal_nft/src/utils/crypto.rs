use anchor_lang::prelude::*;

/// Generate a unique message ID from source chain and address
pub fn generate_message_id(source_chain: u8, source_address: &[u8; 32]) -> [u8; 32] {
    let mut message_id = [0u8; 32];
    message_id[0] = source_chain;
    message_id[1..33].copy_from_slice(source_address);
    message_id
}

/// Verify cross-chain message signature (placeholder for now)
pub fn verify_message_signature(
    _message: &[u8],
    _signature: &[u8; 64],
    _public_key: &[u8; 32]
) -> Result<bool> {
    // TODO: Implement actual signature verification logic
    // For now, return true as placeholder
    Ok(true)
}

/// Hash cross-chain message data
pub fn hash_message_data(data: &[u8]) -> [u8; 32] {
    use anchor_lang::solana_program::keccak::hash;
    let hash_result = hash(data);
    hash_result.to_bytes()
}
