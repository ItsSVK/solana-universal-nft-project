use anchor_lang::prelude::*;

#[error_code]
pub enum UniversalNftError {
    #[msg("Program state already initialized")]
    ProgramStateAlreadyInitialized,
    
    #[msg("Unauthorized operation")]
    Unauthorized,
    
    #[msg("Invalid input")]
    InvalidInput,
    
    #[msg("Account not found")]
    AccountNotFound,
    
    #[msg("NFT already exists")]
    NftAlreadyExists,
    
    #[msg("Collection not verified")]
    CollectionNotVerified,
    
    #[msg("Invalid cross-chain message")]
    InvalidCrossChainMessage,
    
    #[msg("Message already processed")]
    MessageAlreadyProcessed,
    
    #[msg("Invalid token ID")]
    InvalidTokenId,
    
    #[msg("Token supply exceeded")]
    TokenSupplyExceeded,
    
    #[msg("Invalid metadata")]
    InvalidMetadata,
    
    #[msg("PDA derivation failed")]
    PdaDerivationFailed,
    
    #[msg("Insufficient funds")]
    InsufficientFunds,
    
    #[msg("Invalid bump seed")]
    InvalidBumpSeed,
}
