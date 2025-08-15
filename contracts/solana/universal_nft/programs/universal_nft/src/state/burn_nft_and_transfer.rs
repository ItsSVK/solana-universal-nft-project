use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount, Mint};
use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(
    destination_chain: u8,
    recipient_address: [u8; 32],
)]
pub struct BurnNftAndTransfer<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,
    
    /// The NFT mint to be burned
    #[account(
        constraint = mint.decimals == 0 @ crate::error::UniversalNftError::InvalidNftMint,
        constraint = mint.supply == 1 @ crate::error::UniversalNftError::InvalidNftMint,
    )]
    pub mint: Account<'info, Mint>,
    
    /// The owner's token account containing the NFT
    #[account(
        mut,
        constraint = token_account.owner == owner.key() @ crate::error::UniversalNftError::Unauthorized,
        constraint = token_account.mint == mint.key() @ crate::error::UniversalNftError::TokenMintMismatch,
        constraint = token_account.amount == 1 @ crate::error::UniversalNftError::InvalidTokenAmount,
    )]
    pub token_account: Account<'info, TokenAccount>,
    
    /// The NFT origin PDA to extract token ID and metadata
    #[account(
        seeds = [
            NFT_ORIGIN_SEED,
            &nft_origin.token_id.to_le_bytes(),
        ],
        bump,
        constraint = nft_origin.mint_address == mint.key() @ crate::error::UniversalNftError::MintAddressMismatch,
    )]
    pub nft_origin: Account<'info, NftOrigin>,
    
    /// The ZetaChain Gateway program
    /// CHECK: This is the Gateway program that we're calling via CPI
    pub gateway_program: UncheckedAccount<'info>,
    
    /// The Gateway state account
    /// CHECK: Validated by Gateway program
    pub gateway_state: UncheckedAccount<'info>,
    
    /// The Gateway custody token account (ZETA tokens for fees)
    /// CHECK: Validated by Gateway program
    #[account(mut)]
    pub gateway_custody: UncheckedAccount<'info>,
    
    /// The TSS (Threshold Signature Scheme) account for cross-chain operations
    /// CHECK: Validated by Gateway program
    pub tss_account: UncheckedAccount<'info>,
    
    /// The owner's ZETA token account for Gateway fees
    /// CHECK: Validated by Gateway program
    #[account(mut)]
    pub owner_zeta_account: UncheckedAccount<'info>,
    
    /// The Token Program for burning operations
    pub token_program: Program<'info, Token>,
    
    /// The Associated Token Account Program for closing token accounts
    pub associated_token_program: Program<'info, anchor_spl::associated_token::AssociatedToken>,
    
    /// The System Program for closing accounts
    pub system_program: Program<'info, System>,
}
