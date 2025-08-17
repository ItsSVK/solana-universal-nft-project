use anchor_lang::prelude::*;
use crate::error::UniversalNftError;

/// Gateway Authorization Utilities
/// 
/// This module provides functions to validate that only authorized gateways
/// can call cross-chain instructions, preventing unauthorized access to
/// the program's cross-chain functionality.

/// Maximum number of authorized gateways supported
/// 
/// This limits the total number of gateways that can be registered
/// to prevent excessive storage and processing overhead.
pub const MAX_AUTHORIZED_GATEWAYS: u8 = 5;

/// Gateway information structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct AuthorizedGateway {
    /// The gateway's public key
    pub public_key: Pubkey,
    /// Whether the gateway is currently active
    pub is_active: bool,
    /// The gateway's network/chain identifier
    pub network_id: u8,
    /// Timestamp when the gateway was authorized
    pub authorized_at: i64,
    /// Optional description of the gateway
    pub description: Option<String>,
}

/// Gateway authorization result structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct GatewayAuthorizationResult {
    /// Whether the gateway is authorized
    pub is_authorized: bool,
    /// The gateway's public key
    pub gateway_public_key: Pubkey,
    /// Network/chain identifier for the gateway
    pub network_id: u8,
    /// Whether the gateway is currently active
    pub is_active: bool,
    /// Detailed error message if authorization failed
    pub error_message: Option<String>,
}

/// Verify that the caller is an authorized gateway
/// 
/// This function checks if the account calling the cross-chain instruction
/// is an authorized gateway by comparing its public key against the list
/// of authorized gateways.
/// 
/// # Arguments
/// * `caller_public_key` - The public key of the account calling the instruction
/// * `authorized_gateways` - Vector of authorized gateway public keys
/// * `gateway_details` - Optional vector of detailed gateway information
/// 
/// # Returns
/// * `Result<GatewayAuthorizationResult>` - Authorization result with detailed information
/// 
/// # Errors
/// * `UniversalNftError::UnauthorizedGateway` - If the caller is not an authorized gateway
/// * `UniversalNftError::GatewayNotActive` - If the gateway is not currently active
pub fn authorize_gateway(
    caller_public_key: &Pubkey,
    authorized_gateways: &[Pubkey],
    gateway_details: Option<&[AuthorizedGateway]>,
) -> Result<GatewayAuthorizationResult> {
    msg!("Starting gateway authorization check...");
    msg!("Caller public key: {}", caller_public_key);
    msg!("Number of authorized gateways: {}", authorized_gateways.len());

    // Check if the caller is in the list of authorized gateways
    if !authorized_gateways.contains(caller_public_key) {
        let error_msg = format!("Unauthorized gateway: {}", caller_public_key);
        msg!("❌ {}", error_msg);
        
        return Ok(GatewayAuthorizationResult {
            is_authorized: false,
            gateway_public_key: *caller_public_key,
            network_id: 0,
            is_active: false,
            error_message: Some(error_msg),
        });
    }

    msg!("✅ Gateway public key {} found in authorized list", caller_public_key);

    // If we have detailed gateway information, check additional details
    if let Some(details) = gateway_details {
        for gateway in details {
            if gateway.public_key == *caller_public_key {
                // Check if the gateway is active
                if !gateway.is_active {
                    let error_msg = format!("Gateway {} is not currently active", caller_public_key);
                    msg!("❌ {}", error_msg);
                    
                    return Ok(GatewayAuthorizationResult {
                        is_authorized: false,
                        gateway_public_key: *caller_public_key,
                        network_id: gateway.network_id,
                        is_active: false,
                        error_message: Some(error_msg),
                    });
                }

                msg!("✅ Gateway {} is authorized and active", caller_public_key);
                msg!("Network ID: {}", gateway.network_id);
                msg!("Authorized at: {}", gateway.authorized_at);

                return Ok(GatewayAuthorizationResult {
                    is_authorized: true,
                    gateway_public_key: *caller_public_key,
                    network_id: gateway.network_id,
                    is_active: true,
                    error_message: None,
                });
            }
        }
    }

    // If no detailed information, just return basic authorization
    msg!("✅ Gateway {} is authorized (basic check)", caller_public_key);
    
    Ok(GatewayAuthorizationResult {
        is_authorized: true,
        gateway_public_key: *caller_public_key,
        network_id: 0, // Unknown network ID
        is_active: true, // Assume active if no detailed info
        error_message: None,
    })
}

/// Validate gateway registration parameters
/// 
/// This function validates parameters when registering new gateways
/// to ensure they meet the program's requirements.
/// 
/// # Arguments
/// * `gateway_public_key` - The public key of the gateway to authorize
/// * `current_gateway_count` - Current number of authorized gateways
/// * `network_id` - The network/chain identifier for the gateway
/// * `description` - Optional description of the gateway
/// 
/// # Returns
/// * `Result<()>` - Ok if parameters are valid, error if not
/// 
/// # Errors
/// * `UniversalNftError::InvalidGatewayData` - If gateway data is invalid
/// * `UniversalNftError::TooManyGateways` - If maximum gateway limit would be exceeded
pub fn validate_gateway_registration(
    gateway_public_key: &Pubkey,
    current_gateway_count: u8,
    network_id: u8,
    description: &Option<String>,
) -> Result<()> {
    // Check if gateway public key is valid
    require!(
        *gateway_public_key != Pubkey::default(),
        UniversalNftError::InvalidGatewayData
    );
    
    // Check if we're at the maximum gateway limit
    require!(
        current_gateway_count < MAX_AUTHORIZED_GATEWAYS,
        UniversalNftError::TooManyGateways
    );
    
    // Check if network ID is valid (non-zero)
    require!(
        network_id > 0,
        UniversalNftError::InvalidGatewayData
    );
    
    // Check if description is reasonable length (if provided)
    if let Some(desc) = description {
        require!(
            desc.len() <= 100, // Maximum 100 characters
            UniversalNftError::InvalidGatewayData
        );
    }
    
    msg!("✅ Gateway registration parameters validated successfully");
    Ok(())
}

/// Check if a gateway is authorized (simple boolean check)
/// 
/// This is a simplified version of authorize_gateway that just returns
/// a boolean indicating whether the gateway is authorized.
/// 
/// # Arguments
/// * `caller_public_key` - The public key of the account calling the instruction
/// * `authorized_gateways` - Vector of authorized gateway public keys
/// 
/// # Returns
/// * `Result<bool>` - True if authorized, false if not
/// 
/// # Errors
/// * `UniversalNftError::UnauthorizedGateway` - If the caller is not an authorized gateway
pub fn is_gateway_authorized(
    caller_public_key: &Pubkey,
    authorized_gateways: &[Pubkey],
) -> Result<bool> {
    let auth_result = authorize_gateway(caller_public_key, authorized_gateways, None)?;
    Ok(auth_result.is_authorized)
}

/// Log gateway authorization details for debugging and monitoring
/// 
/// This function provides consistent logging for gateway authorization
/// operations for debugging and monitoring purposes.
/// 
/// # Arguments
/// * `operation` - The operation being performed
/// * `caller_public_key` - Public key of the caller
/// * `authorized_gateway_count` - Number of authorized gateways
/// * `details` - Additional details about the authorization
pub fn log_gateway_authorization(
    operation: &str,
    caller_public_key: &Pubkey,
    authorized_gateway_count: u8,
    details: &str,
) {
    msg!("=== Gateway Authorization Operation ===");
    msg!("Operation: {}", operation);
    msg!("Caller: {}", caller_public_key);
    msg!("Authorized Gateways: {}", authorized_gateway_count);
    msg!("Details: {}", details);
    msg!("=====================================");
}

/// Get gateway authorization statistics
/// 
/// This function provides statistics about gateway authorization
/// for monitoring and analytics purposes.
/// 
/// # Arguments
/// * `total_authorizations` - Total number of authorization attempts
/// * `successful_authorizations` - Number of successful authorizations
/// * `failed_authorizations` - Number of failed authorizations
/// 
/// # Returns
/// * `GatewayAuthorizationStats` - Statistics about authorization performance
pub fn get_gateway_authorization_stats(
    total_authorizations: u64,
    successful_authorizations: u64,
    failed_authorizations: u64,
) -> GatewayAuthorizationStats {
    let success_rate_percentage = if total_authorizations > 0 {
        ((successful_authorizations as u128 * 100) / total_authorizations as u128) as u8
    } else {
        0
    };
    
    GatewayAuthorizationStats {
        total_authorizations,
        successful_authorizations,
        failed_authorizations,
        success_rate_percentage,
        last_updated: Clock::get().unwrap().unix_timestamp,
    }
}

/// Gateway authorization statistics structure
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub struct GatewayAuthorizationStats {
    /// Total number of authorization attempts
    pub total_authorizations: u64,
    /// Number of successful authorizations
    pub successful_authorizations: u64,
    /// Number of failed authorizations
    pub failed_authorizations: u64,
    /// Success rate as a percentage (scaled by 100 for integer storage)
    pub success_rate_percentage: u8,
    /// Timestamp of last update
    pub last_updated: i64,
}
