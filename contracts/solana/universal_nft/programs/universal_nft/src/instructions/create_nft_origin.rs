use anchor_lang::prelude::*;
use crate::state::*;
use crate::utils::*;

pub fn create_nft_origin_handler(
    ctx: Context<CreateNftOrigin>,
    token_id: u64,
    origin_chain: u8,
    origin_address: [u8; 32],
    mint_address: Pubkey,
    metadata_uri: String,
) -> Result<()> {
    // Validate inputs
    validate_uri(&metadata_uri)?;
    
    // Get current timestamp
    let clock = Clock::get()?;
    
    // Initialize the NFT origin account
    let nft_origin = &mut ctx.accounts.nft_origin;
    nft_origin.bump = ctx.bumps.nft_origin;
    nft_origin.token_id = token_id;
    nft_origin.origin_chain = origin_chain;
    nft_origin.origin_address = origin_address;
    nft_origin.mint_address = mint_address;
    nft_origin.created_at = clock.unix_timestamp;
    nft_origin.metadata_uri = metadata_uri;
    
    // Validate the created NFT origin data
    validate_nft_origin_data(nft_origin)?;
    
    // Increment the token ID counter in program state
    let program_state = &mut ctx.accounts.program_state;
    let _next_token_id = get_next_token_id(program_state);
    
    msg!("NFT Origin created for token ID: {}", token_id);
    msg!("Origin Chain: {}", origin_chain);
    msg!("Mint Address: {}", mint_address);
    
    Ok(())
}
