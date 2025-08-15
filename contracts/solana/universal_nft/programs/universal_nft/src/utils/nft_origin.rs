use anchor_lang::prelude::*;
use crate::state::NftOrigin;
use crate::utils::pda::derive_nft_origin_pda;

/// Validate and fetch NFT origin account by token_id
pub fn validate_nft_origin_account(
    program_id: &Pubkey,
    token_id: u64,
    nft_origin_account: &Account<NftOrigin>,
) -> Result<()> {
    let (expected_pda, _bump) = derive_nft_origin_pda(program_id, token_id);
    
    // Verify that the provided account is the correct PDA for this token_id
    require!(
        nft_origin_account.key() == expected_pda,
        ErrorCode::InvalidNftOriginAccount
    );
    
    // Verify the account data matches the expected token_id
    require!(
        nft_origin_account.token_id == token_id,
        ErrorCode::TokenIdMismatch
    );
    
    Ok(())
}

/// Verify if a given mint public key matches the original mint stored in the NFT origin account
pub fn verify_mint_matches_origin(
    nft_origin: &NftOrigin,
    mint_to_verify: &Pubkey,
) -> bool {
    nft_origin.mint_address == *mint_to_verify
}

/// Check if a token has been bridged from its original chain
pub fn is_token_bridged_from_chain(
    nft_origin: &NftOrigin,
    expected_chain: u8,
) -> bool {
    nft_origin.origin_chain == expected_chain
}

/// Get the origin chain information for a token
pub fn get_token_origin_chain(nft_origin: &NftOrigin) -> u8 {
    nft_origin.origin_chain
}

/// Get the origin address for a token
pub fn get_token_origin_address(nft_origin: &NftOrigin) -> [u8; 32] {
    nft_origin.origin_address
}

/// Check if an NFT origin account exists for a given token ID
pub fn nft_origin_exists(
    program_id: &Pubkey,
    token_id: u64,
    nft_origin_account: &Account<NftOrigin>,
) -> bool {
    let (expected_pda, _bump) = derive_nft_origin_pda(program_id, token_id);
    
    // Check if the account key matches the expected PDA
    if nft_origin_account.key() != expected_pda {
        return false;
    }
    
    // Check if the account data is valid (non-zero token_id)
    nft_origin_account.token_id > 0
}

/// Validate that an NFT origin account has valid data
pub fn validate_nft_origin_data(nft_origin: &NftOrigin) -> Result<()> {
    // Check that token_id is not zero
    require!(nft_origin.token_id > 0, ErrorCode::InvalidTokenId);
    
    // Check that mint_address is not the default (zero) address
    require!(
        nft_origin.mint_address != Pubkey::default(),
        ErrorCode::InvalidMintAddress
    );
    
    // Check that metadata_uri is not empty
    require!(
        !nft_origin.metadata_uri.is_empty(),
        ErrorCode::InvalidMetadataUri
    );
    
    // Check that created_at is not zero
    require!(nft_origin.created_at > 0, ErrorCode::InvalidTimestamp);
    
    Ok(())
}

/// Get NFT origin information as a formatted string for logging
pub fn format_nft_origin_info(nft_origin: &NftOrigin) -> String {
    format!(
        "NFT Origin - Token ID: {}, Chain: {}, Mint: {}, Created: {}, URI: {}",
        nft_origin.token_id,
        nft_origin.origin_chain,
        nft_origin.mint_address,
        nft_origin.created_at,
        nft_origin.metadata_uri
    )
}

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid token ID")]
    InvalidTokenId,
    #[msg("Invalid mint address")]
    InvalidMintAddress,
    #[msg("Invalid metadata URI")]
    InvalidMetadataUri,
    #[msg("Invalid timestamp")]
    InvalidTimestamp,
    #[msg("Invalid NFT origin account")]
    InvalidNftOriginAccount,
    #[msg("Token ID mismatch")]
    TokenIdMismatch,
}
