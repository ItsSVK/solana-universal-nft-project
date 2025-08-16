use anchor_lang::prelude::*;

#[error_code]
pub enum UniversalNftError {
    #[msg("Invalid program state")]
    InvalidProgramState,
    
    #[msg("Program state already initialized")]
    ProgramStateAlreadyInitialized,
    
    #[msg("Invalid NFT origin data")]
    InvalidNftOriginData,
    
    #[msg("Message already processed")]
    MessageAlreadyProcessed,
    
    #[msg("Invalid NFT mint - must have 0 decimals and supply of 1")]
    InvalidNftMint,
    
    #[msg("Unauthorized operation")]
    Unauthorized,
    
    #[msg("Token mint mismatch")]
    TokenMintMismatch,
    
    #[msg("Invalid token amount - must be exactly 1 for NFTs")]
    InvalidTokenAmount,
    
    #[msg("Mint address mismatch with NFT origin")]
    MintAddressMismatch,
    
    #[msg("Invalid destination chain")]
    InvalidDestinationChain,
    
    #[msg("Invalid recipient address")]
    InvalidRecipientAddress,
    
    #[msg("Gateway call failed")]
    GatewayCallFailed,
    
    #[msg("Invalid cross-chain payload")]
    InvalidCrossChainPayload,
    
    #[msg("Invalid collection data - name, symbol, or URI is invalid")]
    InvalidCollectionData,
    
    #[msg("Collection NFT already minted")]
    CollectionAlreadyMinted,
    
    #[msg("Unsupported chain ID")]
    UnsupportedChainId,
    
    #[msg("Invalid chain ID format")]
    InvalidChainIdFormat,
    
    #[msg("Collection not minted yet")]
    CollectionNotMinted,
    
    #[msg("Collection verification failed")]
    CollectionVerificationFailed,
    
    #[msg("Collection not verified yet")]
    CollectionNotVerified,
    
    #[msg("Collection assignment failed")]
    CollectionAssignmentFailed,
}
