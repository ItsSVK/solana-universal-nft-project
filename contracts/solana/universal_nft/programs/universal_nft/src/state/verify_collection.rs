use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::*;
use crate::constants::*;
use crate::error::UniversalNftError;

#[derive(Accounts)]
pub struct VerifyCollection<'info> {
    /// The authority that can verify the collection
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The program state account containing collection information
    #[account(
        mut,
        seeds = [PROGRAM_STATE_SEED],
        bump,
    )]
    pub program_state: Account<'info, ProgramState>,

    /// The Collection NFT mint account
    #[account(
        mut,
        constraint = collection_mint.key() == program_state.collection_mint @ UniversalNftError::MintAddressMismatch,
    )]
    pub collection_mint: Account<'info, Mint>,

    /// The authority's token account for the Collection NFT
    #[account(
        mut,
        associated_token::mint = collection_mint,
        associated_token::authority = authority,
    )]
    pub authority_token_account: Account<'info, TokenAccount>,

    /// The Token program
    pub token_program: Program<'info, Token>,

    /// The Associated Token Account program
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// The System program
    pub system_program: Program<'info, System>,

    /// The Rent sysvar
    /// CHECK: This is the Rent sysvar
    pub rent: UncheckedAccount<'info>,
}
