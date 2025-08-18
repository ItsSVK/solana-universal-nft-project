use anchor_lang::prelude::*;

pub mod error;
pub mod state;
pub mod constants;
pub mod instructions;
pub mod utils;

use state::*;
use instructions::*;
use instructions::mint_nft::OnCall;

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
        gateway_message_id: [u8; 32],
    ) -> Result<()> {
        instructions::check_and_mark_message_handler(ctx, chain_id, gateway_message_id)
    }

    pub fn check_and_mark_message_with_metadata(
        ctx: Context<CheckAndMarkMessageWithMetadata>,
        chain_id: [u8; 32],
        gateway_message_id: [u8; 32],
        metadata: Option<ReplayProtectionMetadata>,
    ) -> Result<()> {
        instructions::check_and_mark_message_with_metadata_handler(ctx, chain_id, gateway_message_id, metadata)
    }

    pub fn burn_nft_and_transfer(
        ctx: Context<BurnNftAndTransfer>,
        params: BurnNftAndTransferParams,
    ) -> Result<()> {
        instructions::burn_nft_and_transfer(ctx, params)
    }

    pub fn mint_nft(
        ctx: Context<MintNft>,
        _chain_id: u8,
        _gateway_message_id: [u8; 32],
        params: CrossChainMintParams,
    ) -> Result<()> {
        instructions::mint_nft_handler(ctx, params)
    }

    /// ZetaChain Gateway on_call entrypoint for incoming cross-chain NFT minting
    /// 
    /// This instruction is called by the Gateway when a cross-chain message is received.
    /// It implements the full validation pipeline and NFT minting process.
    pub fn on_call(ctx: Context<OnCall>, payload: Vec<u8>) -> Result<()> {
        instructions::on_call_handler(ctx, payload)
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
