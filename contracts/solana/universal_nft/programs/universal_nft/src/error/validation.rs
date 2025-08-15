use anchor_lang::prelude::*;

#[error_code]
pub enum ValidationError {
    #[msg("String length exceeds maximum")]
    StringTooLong,
    
    #[msg("Invalid URI format")]
    InvalidUri,
    
    #[msg("Invalid collection name")]
    InvalidCollectionName,
    
    #[msg("Invalid collection symbol")]
    InvalidCollectionSymbol,
    
    #[msg("Invalid token supply")]
    InvalidTokenSupply,
    
    #[msg("Invalid message format")]
    InvalidMessageFormat,
    
    #[msg("Invalid chain ID")]
    InvalidChainId,
    
    #[msg("Invalid signature")]
    InvalidSignature,
}
