use anchor_lang::prelude::*;
use crate::utils::nft_origin::*;

pub fn get_nft_origin_handler(
    ctx: Context<crate::GetNftOrigin>,
    token_id: u64,
) -> Result<()> {
    // Use our validation function to verify the NFT origin account
    validate_nft_origin_account(
        ctx.program_id,
        token_id,
        &ctx.accounts.nft_origin,
    )?;
    
    // Get the NFT origin data
    let nft_origin = &ctx.accounts.nft_origin;
    
    // Validate the data
    validate_nft_origin_data(nft_origin)?;
    
    // Log the NFT origin information
    msg!("{}", format_nft_origin_info(nft_origin));
    
    // Log additional details
    msg!("Origin Chain: {}", get_token_origin_chain(nft_origin));
    msg!("Origin Address: {:?}", get_token_origin_address(nft_origin));
    msg!("Mint Address: {}", nft_origin.mint_address);
    msg!("Created At: {}", nft_origin.created_at);
    
    // Example: Check if this token was bridged from Ethereum (chain 1)
    if is_token_bridged_from_chain(nft_origin, 1) {
        msg!("This token was bridged from Ethereum");
    }
    
    // Example: Verify a mint address (in real usage, this would be passed as parameter)
    if verify_mint_matches_origin(nft_origin, &nft_origin.mint_address) {
        msg!("Mint address verification successful");
    }
    
    Ok(())
}
