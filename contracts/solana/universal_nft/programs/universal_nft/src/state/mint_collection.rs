use anchor_lang::prelude::*;
use anchor_spl::token::{Mint, Token, TokenAccount};
use anchor_spl::associated_token::AssociatedToken;
use crate::state::*;
use crate::constants::*;

#[derive(Accounts)]
#[instruction(
    name: String,
    symbol: String,
    uri: String,
)]
pub struct MintCollection<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,

    /// The program state account to update with collection mint
    #[account(
        mut,
        seeds = [PROGRAM_STATE_SEED],
        bump,
    )]
    pub program_state: Account<'info, ProgramState>,

    /// The mint account for the Collection NFT
    #[account(
        init,
        payer = authority,
        mint::decimals = 0,
        mint::authority = authority,
    )]
    pub collection_mint: Account<'info, Mint>,

    /// The authority's token account for the Collection NFT
    #[account(
        init_if_needed,
        payer = authority,
        associated_token::mint = collection_mint,
        associated_token::authority = authority,
    )]
    pub authority_token_account: Account<'info, TokenAccount>,

    /// The Token program for minting operations
    pub token_program: Program<'info, Token>,

    /// The Associated Token Account program
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// The System program for account creation
    pub system_program: Program<'info, System>,

    /// The Rent sysvar
    /// CHECK: This is the Rent sysvar
    pub rent: UncheckedAccount<'info>,
}
