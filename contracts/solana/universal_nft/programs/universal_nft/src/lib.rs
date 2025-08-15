use anchor_lang::prelude::*;

pub mod error;
pub mod state;
pub mod constants;
pub mod instructions;
pub mod utils;

use state::*;

declare_id!("AkcS5k7wDCVddJW2VexMV8MSGXYS9UYW5KXvkk47LZaQ");

#[program]
pub mod universal_nft {
    use super::*;

    pub fn initialize_program_state(ctx: Context<InitializeProgramState>) -> Result<()> {
        instructions::initialize_program_state_handler(ctx)
    }

    // TODO: Add other instruction handlers as they are implemented
    // pub fn create_nft_origin(ctx: Context<CreateNftOrigin>) -> Result<()> {
    //     instructions::create_nft_origin_handler(ctx)
    // }
}

// Keep old Initialize for compatibility (no-op)
#[derive(Accounts)]
pub struct Initialize {}
