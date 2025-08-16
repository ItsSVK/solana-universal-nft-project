use anchor_lang::prelude::*;
use crate::state::ProgramState;
use crate::error::UniversalNftError;

/// Verify that the collection is ready for NFT assignment
/// 
/// This function checks if the collection has been minted and verified
/// before allowing NFTs to be assigned to it.
/// 
/// # Arguments
/// * `program_state` - The program state account containing collection information
/// 
/// # Returns
/// * `Result<()>` - Ok if collection is ready, error if not
/// 
/// # Errors
/// * `UniversalNftError::CollectionNotMinted` - If the collection hasn't been minted yet
/// * `UniversalNftError::CollectionNotVerified` - If the collection hasn't been verified yet
pub fn verify_collection_ready_for_assignment(program_state: &Account<ProgramState>) -> Result<()> {
    // Check if collection has been minted
    require!(
        program_state.collection_mint != Pubkey::default(),
        UniversalNftError::CollectionNotMinted
    );

    // Check if collection has been verified
    require!(
        program_state.collection_verified,
        UniversalNftError::CollectionNotVerified
    );

    msg!("Collection verified and ready for NFT assignment");
    Ok(())
}

/// Get the collection mint address from program state
/// 
/// This function safely retrieves the collection mint address
/// after verifying the collection is ready.
/// 
/// # Arguments
/// * `program_state` - The program state account
/// 
/// # Returns
/// * `Result<Pubkey>` - The collection mint address if ready
/// 
/// # Errors
/// * `UniversalNftError::CollectionNotMinted` - If collection not minted
/// * `UniversalNftError::CollectionNotVerified` - If collection not verified
pub fn get_collection_mint_for_assignment(program_state: &Account<ProgramState>) -> Result<Pubkey> {
    verify_collection_ready_for_assignment(program_state)?;
    Ok(program_state.collection_mint)
}

/// Validate collection assignment parameters
/// 
/// This function validates that the collection assignment parameters
/// are correct and consistent.
/// 
/// # Arguments
/// * `collection_mint` - The collection mint address
/// * `nft_mint` - The NFT mint address to be assigned
/// * `collection_metadata` - The collection metadata account
/// * `nft_metadata` - The NFT metadata account
/// 
/// # Returns
/// * `Result<()>` - Ok if parameters are valid, error if not
/// 
/// # Errors
/// * `UniversalNftError::InvalidCollectionData` - If collection data is invalid
/// * `UniversalNftError::MintAddressMismatch` - If mint addresses don't match
pub fn validate_collection_assignment_params(
    collection_mint: &Pubkey,
    nft_mint: &Pubkey,
    collection_metadata: &Pubkey,
    nft_metadata: &Pubkey,
) -> Result<()> {
    // Ensure collection mint is not the same as NFT mint
    require!(
        collection_mint != nft_mint,
        UniversalNftError::InvalidCollectionData
    );

    // Ensure addresses are valid (not default)
    require!(
        *collection_mint != Pubkey::default(),
        UniversalNftError::InvalidCollectionData
    );

    require!(
        *nft_mint != Pubkey::default(),
        UniversalNftError::InvalidCollectionData
    );

    require!(
        *collection_metadata != Pubkey::default(),
        UniversalNftError::InvalidCollectionData
    );

    require!(
        *nft_metadata != Pubkey::default(),
        UniversalNftError::InvalidCollectionData
    );

    msg!("Collection assignment parameters validated successfully");
    Ok(())
}

/// Log collection assignment information
/// 
/// This function logs detailed information about the collection assignment
/// for debugging and monitoring purposes.
/// 
/// # Arguments
/// * `collection_mint` - The collection mint address
/// * `nft_mint` - The NFT mint address being assigned
/// * `collection_metadata` - The collection metadata account
/// * `nft_metadata` - The NFT metadata account
pub fn log_collection_assignment_info(
    collection_mint: &Pubkey,
    nft_mint: &Pubkey,
    collection_metadata: &Pubkey,
    nft_metadata: &Pubkey,
) {
    msg!("=== Collection Assignment Information ===");
    msg!("Collection Mint: {}", collection_mint);
    msg!("NFT Mint: {}", nft_mint);
    msg!("Collection Metadata: {}", collection_metadata);
    msg!("NFT Metadata: {}", nft_metadata);
    msg!("=========================================");
}
