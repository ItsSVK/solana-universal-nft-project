use anchor_lang::prelude::*;
use crate::constants::*;

/// Derive PDA for program state
pub fn derive_program_state_pda(program_id: &Pubkey) -> (Pubkey, u8) {
    Pubkey::find_program_address(&[PROGRAM_STATE_SEED], program_id)
}

/// Derive PDA for NFT origin
pub fn derive_nft_origin_pda(program_id: &Pubkey, token_id: u64) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[NFT_ORIGIN_SEED, &token_id.to_le_bytes()],
        program_id
    )
}

/// Derive PDA for replay protection
pub fn derive_replay_protection_pda(program_id: &Pubkey, message_hash: &[u8; 32]) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[REPLAY_SEED, message_hash],
        program_id
    )
}

/// Derive PDA for collection
pub fn derive_collection_pda(program_id: &Pubkey, collection_name: &str) -> (Pubkey, u8) {
    Pubkey::find_program_address(
        &[COLLECTION_SEED, collection_name.as_bytes()],
        program_id
    )
}
