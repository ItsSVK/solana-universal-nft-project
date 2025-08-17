use anchor_lang::prelude::*;
use crate::utils::gateway_authorization::AuthorizedGateway;

#[account]
pub struct ProgramState {
    pub bump: u8,
    pub next_token_id: u64,
    pub collection_mint: Pubkey,
    pub collection_verified: bool,
    /// List of authorized gateways that can call cross-chain instructions
    pub authorized_gateways: Vec<AuthorizedGateway>,
    /// ZetaChain Gateway program ID for validation
    pub zeta_chain_gateway_program_id: Pubkey,
    /// Whether Gateway integration is currently active
    pub gateway_integration_active: bool,
    /// Timestamp when Gateway integration was last updated
    pub gateway_last_updated: i64,
}

impl ProgramState {
    pub const LEN: usize = 8 + // discriminator
                           1 + // bump
                           8 + // next_token_id
                           32 + // collection_mint
                           1 + // collection_verified
                           4 + // authorized_gateways vector length
                           (32 + 1 + 1 + 8 + 4) * 5 + // authorized_gateways data (max 5 gateways)
                           32 + // zeta_chain_gateway_program_id
                           1 + // gateway_integration_active
                           8; // gateway_last_updated
}

impl ProgramState {
    /// Check if a gateway is authorized
    pub fn is_gateway_authorized(&self, gateway_pubkey: &Pubkey) -> bool {
        self.authorized_gateways.iter().any(|gateway| {
            gateway.public_key == *gateway_pubkey && gateway.is_active
        })
    }

    /// Add a new authorized gateway
    pub fn add_authorized_gateway(&mut self, gateway: AuthorizedGateway) -> Result<()> {
        require!(
            self.authorized_gateways.len() < 5,
            crate::error::UniversalNftError::TooManyGateways
        );
        
        // Check if gateway already exists
        if self.authorized_gateways.iter().any(|g| g.public_key == gateway.public_key) {
            return Err(crate::error::UniversalNftError::InvalidGatewayData.into());
        }
        
        self.authorized_gateways.push(gateway);
        self.gateway_last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Remove an authorized gateway
    pub fn remove_authorized_gateway(&mut self, gateway_pubkey: &Pubkey) -> Result<()> {
        let initial_len = self.authorized_gateways.len();
        self.authorized_gateways.retain(|g| g.public_key != *gateway_pubkey);
        
        if self.authorized_gateways.len() == initial_len {
            return Err(crate::error::UniversalNftError::InvalidGatewayData.into());
        }
        
        self.gateway_last_updated = Clock::get()?.unix_timestamp;
        Ok(())
    }

    /// Update Gateway integration status
    pub fn update_gateway_integration(&mut self, active: bool) {
        self.gateway_integration_active = active;
        self.gateway_last_updated = Clock::get().unwrap().unix_timestamp;
    }

    /// Get the number of authorized gateways
    pub fn authorized_gateway_count(&self) -> usize {
        self.authorized_gateways.len()
    }

    /// Check if Gateway integration is ready
    pub fn is_gateway_ready(&self) -> bool {
        self.gateway_integration_active && !self.authorized_gateways.is_empty()
    }
}

pub fn get_next_token_id(program_state: &mut Account<ProgramState>) -> u64 {
    let next = program_state.next_token_id;
    program_state.next_token_id = program_state
        .next_token_id
        .checked_add(1)
        .expect("token id overflow");
    next
}
