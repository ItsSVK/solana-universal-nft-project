use anchor_lang::prelude::*;

/// NFT Origin account storing cross-chain origin data
#[account]
pub struct NftOrigin {
    /// Bump seed for the PDA
    pub bump: u8,
    /// Token ID from the source chain
    pub token_id: u64,
    /// Origin chain ID
    pub origin_chain: u8,
    /// Origin address (32 bytes)
    pub origin_address: [u8; 32],
    /// Mint address on Solana
    pub mint_address: Pubkey,
    /// Timestamp when created
    pub created_at: i64,
    /// Metadata URI
    pub metadata_uri: String,
}

/// NFT Origin by Token ID mapping for efficient lookups
/// This PDA maps token_id to mint_address for quick existence checks
#[account]
pub struct NftOriginByTokenId {
    /// Bump seed for the PDA
    pub bump: u8,
    /// Token ID from the source chain (used as seed)
    pub token_id: u64,
    /// Mint address on Solana
    pub mint_address: Pubkey,
    /// Timestamp when created
    pub created_at: i64,
}

impl NftOrigin {
    pub const LEN: usize = 8 + 1 + 8 + 1 + 32 + 32 + 8 + 200; // discriminator + fields

    /// Initialize a new NftOrigin account
    pub fn initialize(
        &mut self,
        bump: u8,
        token_id: u64,
        origin_chain: u8,
        origin_address: [u8; 32],
        mint_address: Pubkey,
        metadata_uri: String,
    ) -> Result<()> {
        self.bump = bump;
        self.token_id = token_id;
        self.origin_chain = origin_chain;
        self.origin_address = origin_address;
        self.mint_address = mint_address;
        self.created_at = Clock::get()?.unix_timestamp;
        self.metadata_uri = metadata_uri;
        Ok(())
    }
}

impl NftOriginByTokenId {
    pub const LEN: usize = 8 + 1 + 8 + 32 + 8; // discriminator + fields

    /// Initialize a new NftOriginByTokenId account
    pub fn initialize(
        &mut self,
        bump: u8,
        token_id: u64,
        mint_address: Pubkey,
    ) -> Result<()> {
        self.bump = bump;
        self.token_id = token_id;
        self.mint_address = mint_address;
        self.created_at = Clock::get()?.unix_timestamp;
        Ok(())
    }
}
