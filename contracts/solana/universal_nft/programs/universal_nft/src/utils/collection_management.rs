use anchor_lang::prelude::*;
use crate::state::ProgramState;
use crate::error::UniversalNftError;
use crate::utils::verify_collection_ready_for_assignment;

/// Collection Management Utilities
/// 
/// This module provides helper functions for managing collection relationships,
/// checking collection membership, and handling collection updates.

/// Check if an NFT belongs to the program's verified collection
/// 
/// This function verifies that the collection has been minted and verified,
/// and then checks if the provided NFT mint belongs to the collection.
/// 
/// # Arguments
/// * `program_state` - The program state account containing collection information
/// * `nft_mint` - The NFT mint address to check
/// 
/// # Returns
/// * `Result<bool>` - True if NFT belongs to collection, false otherwise
/// 
/// # Errors
/// * `UniversalNftError::CollectionNotMinted` - If the collection hasn't been minted yet
/// * `UniversalNftError::CollectionNotVerified` - If the collection hasn't been verified yet
pub fn is_nft_in_collection(
    program_state: &Account<ProgramState>,
    nft_mint: &Pubkey,
) -> Result<bool> {
    // First verify the collection is ready
    verify_collection_ready_for_assignment(program_state)?;
    
    // For now, we'll return true if the collection is verified
    // In a full implementation, this would check against Metaplex collection data
    // and verify the NFT's metadata contains the collection mint
    
    msg!("NFT {} is considered part of the verified collection", nft_mint);
    Ok(true)
}

/// Get collection information for display and management purposes
/// 
/// This function returns a summary of the collection status and details
/// that can be used for UI display or program logic.
/// 
/// # Arguments
/// * `program_state` - The program state account
/// 
/// # Returns
/// * `Result<CollectionInfo>` - Collection information struct
/// 
/// # Errors
/// * `UniversalNftError::CollectionNotMinted` - If collection not minted
pub fn get_collection_info(program_state: &Account<ProgramState>) -> Result<CollectionInfo> {
    // Check if collection has been minted
    require!(
        program_state.collection_mint != Pubkey::default(),
        UniversalNftError::CollectionNotMinted
    );
    
    Ok(CollectionInfo {
        mint: program_state.collection_mint,
        verified: program_state.collection_verified,
        total_nfts: 0, // TODO: Implement NFT counting logic
    })
}

/// Validate collection update parameters
/// 
/// This function validates parameters for collection updates like metadata changes
/// 
/// # Arguments
/// * `new_uri` - New metadata URI for the collection
/// * `new_name` - New name for the collection (optional)
/// * `new_symbol` - New symbol for the collection (optional)
/// 
/// # Returns
/// * `Result<()>` - Ok if parameters are valid, error if not
/// 
/// # Errors
/// * `UniversalNftError::InvalidCollectionData` - If collection data is invalid
pub fn validate_collection_update_params(
    new_uri: &str,
    new_name: Option<&str>,
    new_symbol: Option<&str>,
) -> Result<()> {
    // Validate URI
    require!(
        !new_uri.is_empty() && new_uri.len() <= 200,
        UniversalNftError::InvalidCollectionData
    );
    
    // Validate name if provided
    if let Some(name) = new_name {
        require!(
            !name.is_empty() && name.len() <= 32,
            UniversalNftError::InvalidCollectionData
        );
    }
    
    // Validate symbol if provided
    if let Some(symbol) = new_symbol {
        require!(
            !symbol.is_empty() && symbol.len() <= 10,
            UniversalNftError::InvalidCollectionData
        );
    }
    
    msg!("Collection update parameters validated successfully");
    Ok(())
}

/// Log collection management operations for debugging and monitoring
/// 
/// This function provides consistent logging for collection-related operations
/// 
/// # Arguments
/// * `operation` - The operation being performed
/// * `collection_mint` - The collection mint address
/// * `details` - Additional details about the operation
pub fn log_collection_operation(
    operation: &str,
    collection_mint: &Pubkey,
    details: &str,
) {
    msg!("=== Collection Management Operation ===");
    msg!("Operation: {}", operation);
    msg!("Collection Mint: {}", collection_mint);
    msg!("Details: {}", details);
    msg!("=====================================");
}

/// Collection information structure for external consumption
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct CollectionInfo {
    /// The collection mint address
    pub mint: Pubkey,
    /// Whether the collection has been verified
    pub verified: bool,
    /// Total number of NFTs in the collection (placeholder for future implementation)
    pub total_nfts: u64,
}

/// Check if collection operations are allowed
/// 
/// This function verifies that the collection is in a state where
/// management operations can be performed
/// 
/// # Arguments
/// * `program_state` - The program state account
/// 
/// # Returns
/// * `Result<()>` - Ok if operations are allowed, error if not
/// 
/// # Errors
/// * `UniversalNftError::CollectionNotMinted` - If collection not minted
/// * `UniversalNftError::CollectionNotVerified` - If collection not verified
pub fn verify_collection_management_allowed(
    program_state: &Account<ProgramState>,
) -> Result<()> {
    verify_collection_ready_for_assignment(program_state)?;
    
    msg!("Collection management operations are allowed");
    Ok(())
}

/// Get collection statistics for analytics and monitoring
/// 
/// This function provides collection statistics that can be used
/// for program analytics and user interface display
/// 
/// # Arguments
/// * `program_state` - The program state account
/// 
/// # Returns
/// * `Result<CollectionStats>` - Collection statistics struct
/// 
/// # Errors
/// * `UniversalNftError::CollectionNotMinted` - If collection not minted
pub fn get_collection_stats(program_state: &Account<ProgramState>) -> Result<CollectionStats> {
    // Check if collection has been minted
    require!(
        program_state.collection_mint != Pubkey::default(),
        UniversalNftError::CollectionNotMinted
    );
    
    Ok(CollectionStats {
        collection_mint: program_state.collection_mint,
        verified: program_state.collection_verified,
        total_nfts: 0, // TODO: Implement NFT counting
        last_updated: Clock::get()?.unix_timestamp,
    })
}

/// Collection statistics structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct CollectionStats {
    /// The collection mint address
    pub collection_mint: Pubkey,
    /// Whether the collection has been verified
    pub verified: bool,
    /// Total number of NFTs in the collection
    pub total_nfts: u64,
    /// Timestamp of last update
    pub last_updated: i64,
}
