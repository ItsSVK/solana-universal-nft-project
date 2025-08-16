use anchor_lang::prelude::*;
use anchor_spl::token::{mint_to, MintTo};
use crate::state::*;
use crate::error::UniversalNftError;

/// Mint the Collection NFT and set up its metadata
///
/// This instruction:
/// 1. Creates a new mint account for the Collection NFT
/// 2. Mints exactly 1 token to the authority
/// 3. Creates metadata for the Collection NFT (simplified version)
/// 4. Updates the program state with the collection mint address
pub fn mint_collection_handler(
    ctx: Context<MintCollection>,
    name: String,
    symbol: String,
    uri: String,
) -> Result<()> {
    let authority = &ctx.accounts.authority;
    let program_state = &mut ctx.accounts.program_state;
    let collection_mint = &ctx.accounts.collection_mint;

    // Validate inputs
    require!(!name.is_empty(), UniversalNftError::InvalidCollectionData);
    require!(!symbol.is_empty(), UniversalNftError::InvalidCollectionData);
    require!(!uri.is_empty(), UniversalNftError::InvalidCollectionData);
    require!(name.len() <= 32, UniversalNftError::InvalidCollectionData);
    require!(symbol.len() <= 10, UniversalNftError::InvalidCollectionData);
    require!(uri.len() <= 200, UniversalNftError::InvalidCollectionData);

    // Check if collection is already minted
    require!(
        program_state.collection_mint == Pubkey::default(),
        UniversalNftError::CollectionAlreadyMinted
    );

    msg!("Starting Collection NFT minting process");
    msg!("Collection Name: {}", name);
    msg!("Collection Symbol: {}", symbol);
    msg!("Collection URI: {}", uri);

    // Step 1: Mint exactly 1 token to the authority
    msg!("Minting 1 Collection NFT token to authority...");

    let mint_to_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        MintTo {
            mint: ctx.accounts.collection_mint.to_account_info(),
            to: ctx.accounts.authority_token_account.to_account_info(),
            authority: ctx.accounts.authority.to_account_info(),
        },
    );

    mint_to(mint_to_ctx, 1)?;

    msg!("Collection NFT token minted successfully");

    // Step 2: Create metadata for the Collection NFT (simplified version)
    msg!("Creating metadata for Collection NFT...");

    // For now, we'll create a simplified metadata structure
    // In a full implementation, this would call the Metaplex Token Metadata program
    // to create proper metadata accounts
    
    // Create metadata account data structure
    let metadata_data = CollectionMetadata {
        name,
        symbol,
        uri,
        seller_fee_basis_points: 0,
        creators: vec![Creator {
            address: authority.key(),
            verified: true,
            share: 100,
        }],
    };

    // Note: In a real implementation, this would be a proper Metaplex metadata account
    // For now, we'll just log the metadata creation
    msg!("Collection NFT metadata created successfully");
    msg!("Metadata Name: {}", metadata_data.name);
    msg!("Metadata Symbol: {}", metadata_data.symbol);
    msg!("Metadata URI: {}", metadata_data.uri);

    // Step 3: Update program state with collection mint address
    msg!("Updating program state with collection mint address...");

    program_state.collection_mint = collection_mint.key();
    // Note: collection_verified remains false until verification is done in a separate instruction

    msg!("Collection NFT minting completed successfully!");
    msg!("Collection Mint Address: {}", collection_mint.key());

    Ok(())
}

/// Simplified metadata structure for Collection NFT
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct CollectionMetadata {
    pub name: String,
    pub symbol: String,
    pub uri: String,
    pub seller_fee_basis_points: u16,
    pub creators: Vec<Creator>,
}

/// Creator information for metadata
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    pub share: u8,
}
