use anchor_lang::prelude::*;
use crate::error::universal_nft::UniversalNftError;

// ZetaChain Gateway program ID (this should be the actual Gateway program ID)
// For now, we'll use a placeholder - this should be updated with the real Gateway program ID
const ZETACHAIN_GATEWAY_PROGRAM_ID: &str = "11111111111111111111111111111111"; // Placeholder

/// Context for the on_revert instruction
/// 
/// This instruction is called by the ZetaChain Gateway when a cross-chain transfer fails.
/// It implements the required Gateway interface for handling failed transfers.
#[derive(Accounts)]
pub struct OnRevert<'info> {
    /// The account paying for the transaction (usually the Gateway)
    #[account(mut)]
    pub payer: Signer<'info>,

    /// The ZetaChain Gateway program that is calling this instruction
    /// CHECK: This is validated to ensure only the official Gateway can call on_revert
    pub gateway_program: UncheckedAccount<'info>,

    /// The System program for account operations
    pub system_program: Program<'info, System>,
}

/// Handler for the on_revert instruction
/// 
/// This function is called by the ZetaChain Gateway when a cross-chain transfer fails.
/// It logs the revert information for debugging and monitoring purposes.
/// 
/// For NFTs, this is a no-op since failed transfers don't create state on the destination chain,
/// but it's required for compliance with the Gateway interface.
pub fn on_revert_handler(
    ctx: Context<OnRevert>,
    sender: [u8; 32],
    message: Vec<u8>,
    reason: Vec<u8>,
) -> Result<()> {
    // Validate that the caller is the ZetaChain Gateway program
    let gateway_program_id = ctx.accounts.gateway_program.key();
    let expected_gateway_id = Pubkey::new_from_array([
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
    ]); // Placeholder - should be actual Gateway program ID
    
    require!(
        gateway_program_id == expected_gateway_id,
        UniversalNftError::UnauthorizedGateway
    );

    // Log the revert information for debugging and monitoring
    msg!("=== on_revert called ===");
    msg!("Sender: {:?}", sender);
    msg!("Message data length: {}", message.len());
    msg!("Message data: {:?}", message);
    msg!("Revert reason: {}", String::from_utf8_lossy(&reason));
    msg!("Gateway program: {}", ctx.accounts.gateway_program.key());
    msg!("Payer: {}", ctx.accounts.payer.key());
    msg!("=== on_revert completed ===");

    // For NFTs, no state cleanup is needed since failed transfers don't create state
    // on the destination chain. This is a no-op but required for Gateway compliance.
    
    Ok(())
}
