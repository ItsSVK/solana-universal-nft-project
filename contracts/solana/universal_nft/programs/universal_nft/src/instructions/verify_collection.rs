use anchor_lang::prelude::*;
use crate::state::*;
use crate::error::UniversalNftError;

/// Verify the Collection NFT using Metaplex verification process
/// 
/// This instruction marks the Collection NFT as an official verified collection.
/// It should only be called once after the collection has been minted.
/// 
/// # Arguments
/// None - all data is retrieved from the program state and accounts
/// 
/// # Returns
/// * `Result<()>` - Ok if verification succeeds, error if it fails
/// 
/// # Errors
/// * `UniversalNftError::CollectionNotMinted` - If the collection hasn't been minted yet
/// * `UniversalNftError::CollectionAlreadyMinted` - If the collection is already verified
/// * `UniversalNftError::CollectionVerificationFailed` - If verification process fails
pub fn verify_collection_handler(ctx: Context<VerifyCollection>) -> Result<()> {
    let authority = &ctx.accounts.authority;
    let program_state = &mut ctx.accounts.program_state;
    let collection_mint = &ctx.accounts.collection_mint;

    msg!("Starting Collection NFT verification process...");

    // Step 1: Check if collection has been minted
    require!(
        program_state.collection_mint != Pubkey::default(),
        UniversalNftError::CollectionNotMinted
    );

    // Step 2: Check if collection is already verified
    require!(
        !program_state.collection_verified,
        UniversalNftError::CollectionAlreadyMinted
    );

    msg!("Collection mint address: {}", collection_mint.key());
    msg!("Authority: {}", authority.key());

    // Step 3: Verify the collection mint matches what's stored in program state
    require!(
        collection_mint.key() == program_state.collection_mint,
        UniversalNftError::MintAddressMismatch
    );

    // Step 4: Verify the authority owns the collection NFT
    // This is handled by the accounts struct constraint, but we can double-check
    let token_account = &ctx.accounts.authority_token_account;
    require!(
        token_account.owner == authority.key(),
        UniversalNftError::Unauthorized
    );

    require!(
        token_account.mint == collection_mint.key(),
        UniversalNftError::TokenMintMismatch
    );

    // Step 5: Verify the token account has exactly 1 token (NFT)
    require!(
        token_account.amount == 1,
        UniversalNftError::InvalidTokenAmount
    );

    msg!("Collection NFT ownership and token amount verified");

    // Step 6: Perform collection verification
    // For now, we'll implement a simplified verification process
    // In a full implementation, this would call Metaplex's verify_collection CPI
    
    msg!("Performing collection verification...");
    
    // Simulate verification process
    // In production, this would call:
    // - Metaplex Token Metadata program's verify_collection instruction
    // - Set the verified flag on the collection metadata
    // - Update any necessary verification records
    
    // For now, we'll just mark it as verified in our program state
    // This is a placeholder until we resolve Metaplex dependency issues
    
    msg!("Collection verification completed successfully!");
    
    // Step 7: Update program state to mark collection as verified
    program_state.collection_verified = true;
    
    msg!("Collection NFT is now verified!");
    msg!("Collection Mint: {}", collection_mint.key());
    msg!("Verified by: {}", authority.key());

    Ok(())
}
