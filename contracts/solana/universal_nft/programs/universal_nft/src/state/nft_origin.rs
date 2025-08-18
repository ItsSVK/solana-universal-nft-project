use anchor_lang::prelude::*;

#[account]
pub struct NftOrigin {
    pub bump: u8,
    pub token_id: u64,
    pub origin_chain: u8,
    pub origin_address: [u8; 32],
    pub mint_address: Pubkey,
    pub created_at: i64,
    pub metadata_uri: String,
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
