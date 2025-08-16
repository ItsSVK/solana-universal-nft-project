/// Supported chain IDs for cross-chain NFT transfers
/// 
/// These chain IDs represent the blockchain networks that are part of our
/// cross-chain NFT ecosystem. Only messages from these chains will be processed.
pub const SUPPORTED_CHAIN_IDS: [u8; 3] = [
    CHAIN_ID_SOLANA,
    CHAIN_ID_BASE_SEPOLIA,
    CHAIN_ID_BNB_TESTNET,
];

/// Chain ID for Solana mainnet/devnet
pub const CHAIN_ID_SOLANA: u8 = 1;

/// Chain ID for Base Sepolia testnet
pub const CHAIN_ID_BASE_SEPOLIA: u8 = 2;

/// Chain ID for BNB Smart Chain testnet
pub const CHAIN_ID_BNB_TESTNET: u8 = 3;

/// Maximum number of supported chains
pub const MAX_SUPPORTED_CHAINS: usize = SUPPORTED_CHAIN_IDS.len();

/// Check if a chain ID is supported
pub fn is_chain_supported(chain_id: u8) -> bool {
    SUPPORTED_CHAIN_IDS.contains(&chain_id)
}

/// Get the number of supported chains
pub fn get_supported_chain_count() -> usize {
    SUPPORTED_CHAIN_IDS.len()
}

/// Get all supported chain IDs as a slice
pub fn get_supported_chain_ids() -> &'static [u8] {
    &SUPPORTED_CHAIN_IDS
}
