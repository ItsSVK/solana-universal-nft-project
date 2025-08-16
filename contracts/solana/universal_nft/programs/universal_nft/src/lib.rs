use anchor_lang::prelude::*;

pub mod error;
pub mod state;
pub mod constants;
pub mod instructions;
pub mod utils;

use state::*;
use instructions::*;

declare_id!("AkcS5k7wDCVddJW2VexMV8MSGXYS9UYW5KXvkk47LZaQ");

#[program]
pub mod universal_nft {
    use super::*;

    pub fn initialize_program_state(ctx: Context<InitializeProgramState>) -> Result<()> {
        instructions::initialize_program_state_handler(ctx)
    }

    pub fn create_nft_origin(
        ctx: Context<CreateNftOrigin>,
        token_id: u64,
        origin_chain: u8,
        origin_address: [u8; 32],
        mint_address: Pubkey,
        metadata_uri: String,
    ) -> Result<()> {
        instructions::create_nft_origin_handler(ctx, token_id, origin_chain, origin_address, mint_address, metadata_uri)
    }

    pub fn get_nft_origin(
        ctx: Context<GetNftOrigin>,
        token_id: u64,
    ) -> Result<()> {
        instructions::get_nft_origin_handler(ctx, token_id)
    }

    pub fn check_and_mark_message(
        ctx: Context<CheckAndMarkMessage>,
        chain_id: [u8; 32],
        message_hash: [u8; 32],
    ) -> Result<()> {
        instructions::check_and_mark_message_handler(ctx, chain_id, message_hash)
    }

    pub fn burn_nft_and_transfer(
        ctx: Context<BurnNftAndTransfer>,
        destination_chain: u8,
        recipient_address: [u8; 32],
    ) -> Result<()> {
        instructions::burn_nft_and_transfer_handler(ctx, destination_chain, recipient_address)
    }

    pub fn close_program_state(ctx: Context<CloseProgramState>) -> Result<()> {
        instructions::close_program_state_handler(ctx)
    }

    pub fn close_nft_origin(ctx: Context<CloseNftOrigin>) -> Result<()> {
        instructions::close_nft_origin_handler(ctx)
    }

    pub fn close_replay_protection(ctx: Context<CloseReplayProtection>) -> Result<()> {
        instructions::close_replay_protection_handler(ctx)
    }

    pub fn mint_collection(
        ctx: Context<MintCollection>,
        name: String,
        symbol: String,
        uri: String,
    ) -> Result<()> {
        instructions::mint_collection_handler(ctx, name, symbol, uri)
    }

    pub fn verify_collection(ctx: Context<VerifyCollection>) -> Result<()> {
        instructions::verify_collection_handler(ctx)
    }
}

// Keep old Initialize for compatibility (no-op)
#[derive(Accounts)]
pub struct Initialize {}
