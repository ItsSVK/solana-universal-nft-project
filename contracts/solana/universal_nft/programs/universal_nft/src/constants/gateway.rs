/// ZetaChain Gateway Constants
/// 
/// This module contains constants related to the ZetaChain Gateway integration,
/// including program IDs, instruction discriminators, and configuration values.

use anchor_lang::prelude::*;

/// ZetaChain Gateway Program ID on Solana devnet
/// 
/// This is the official ZetaChain Gateway program deployed on Solana devnet.
/// For mainnet, this would be the mainnet program ID.
pub const ZETA_CHAIN_GATEWAY_PROGRAM_ID: &str = "ZetaChainGateway111111111111111111111111111111111";

/// ZetaChain Gateway Program ID as Pubkey
/// 
/// Note: This is a placeholder. In production, you would need to get the actual
/// program ID from the ZetaChain Gateway deployment.
pub const ZETA_CHAIN_GATEWAY_PROGRAM_ID_PUBKEY: Pubkey = Pubkey::new_from_array([
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
    0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
]);

/// ZetaChain Gateway instruction discriminators
/// 
/// These are the 8-byte discriminators for Gateway instructions.
/// They are derived from the instruction names using a specific algorithm.

/// Instruction discriminator for `deposit` instruction
pub const GATEWAY_DEPOSIT_DISCRIMINATOR: [u8; 8] = [0x8f, 0x4a, 0x8c, 0x8f, 0x4a, 0x8c, 0x8f, 0x4a]; // Placeholder - replace with actual

/// Instruction discriminator for `deposit_and_call` instruction  
pub const GATEWAY_DEPOSIT_AND_CALL_DISCRIMINATOR: [u8; 8] = [0x9f, 0x5b, 0x9d, 0x9f, 0x5b, 0x9d, 0x9f, 0x5b]; // Placeholder - replace with actual

/// Instruction discriminator for `withdraw` instruction
pub const GATEWAY_WITHDRAW_DISCRIMINATOR: [u8; 8] = [0xaf, 0x6c, 0xae, 0xaf, 0x6c, 0xae, 0xaf, 0x6c]; // Placeholder - replace with actual

/// Gateway configuration constants
pub const DEFAULT_GAS_LIMIT: u64 = 100_000; // Default gas limit for cross-chain operations
pub const DEFAULT_ZETA_FEE_AMOUNT: u64 = 1_000_000; // 0.001 ZETA fee (in lamports)
pub const MAX_MESSAGE_SIZE: usize = 10_000; // Maximum message size in bytes
pub const MAX_DESTINATION_ADDRESS_SIZE: usize = 32; // Maximum destination address size

/// Gateway account validation constants
pub const GATEWAY_STATE_SEED: &[u8] = b"gateway_state";
pub const GATEWAY_CUSTODY_SEED: &[u8] = b"gateway_custody";
pub const TSS_ACCOUNT_SEED: &[u8] = b"tss_account";

/// Gateway error codes
pub const GATEWAY_ERROR_INSUFFICIENT_FUNDS: u32 = 1;
pub const GATEWAY_ERROR_INVALID_DESTINATION: u32 = 2;
pub const GATEWAY_ERROR_MESSAGE_TOO_LARGE: u32 = 3;
pub const GATEWAY_ERROR_INVALID_CHAIN_ID: u32 = 4;

/// Get the ZetaChain Gateway program ID as a Pubkey
pub fn get_gateway_program_id() -> Pubkey {
    ZETA_CHAIN_GATEWAY_PROGRAM_ID_PUBKEY
}

/// Validate that a program ID matches the ZetaChain Gateway
pub fn is_gateway_program(program_id: &Pubkey) -> bool {
    program_id == &ZETA_CHAIN_GATEWAY_PROGRAM_ID_PUBKEY
}

/// Get the instruction discriminator for a specific Gateway instruction
pub fn get_gateway_instruction_discriminator(instruction_name: &str) -> Option<[u8; 8]> {
    match instruction_name {
        "deposit" => Some(GATEWAY_DEPOSIT_DISCRIMINATOR),
        "deposit_and_call" => Some(GATEWAY_DEPOSIT_AND_CALL_DISCRIMINATOR),
        "withdraw" => Some(GATEWAY_WITHDRAW_DISCRIMINATOR),
        _ => None,
    }
}

/// Validate Gateway instruction data format
pub fn validate_gateway_instruction_data(data: &[u8], expected_size: usize) -> bool {
    data.len() >= expected_size
}

/// Calculate Gateway fee based on message size and destination chain
pub fn calculate_gateway_fee(message_size: usize, destination_chain: u8) -> u64 {
    let base_fee = DEFAULT_ZETA_FEE_AMOUNT;
    let size_multiplier = (message_size as u64 / 1000) + 1;
    let chain_multiplier = match destination_chain {
        1 => 1, // Solana
        2 => 2, // Base Sepolia  
        3 => 2, // BNB Testnet
        _ => 3, // Unknown chains cost more
    };
    
    base_fee * size_multiplier * chain_multiplier
}
