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
    
    #[msg("Payload size is too small")]
    PayloadTooSmall,
    
    #[msg("Payload size is too large")]
    PayloadTooLarge,
    
    #[msg("Invalid payload bounds - min must be <= max and within global limits")]
    InvalidPayloadBounds,
    
    #[msg("Invalid signature - signature verification failed")]
    InvalidSignature,
    
    #[msg("Insufficient signatures - not enough valid signatures provided")]
    InsufficientSignatures,
    
    #[msg("Unauthorized validator - signer is not an authorized validator")]
    UnauthorizedValidator,
    
    #[msg("Invalid signature threshold - threshold calculation failed")]
    InvalidSignatureThreshold,
    
    #[msg("Invalid validator data - validator information is invalid")]
    InvalidValidatorData,
    
    #[msg("Too many validators - maximum validator limit exceeded")]
    TooManyValidators,
    
    #[msg("Unauthorized gateway - caller is not an authorized gateway")]
    UnauthorizedGateway,
    
    #[msg("Gateway not active - gateway is authorized but not currently active")]
    GatewayNotActive,
    
    #[msg("Invalid gateway data - gateway information is invalid")]
    InvalidGatewayData,
    
    #[msg("Too many gateways - maximum gateway limit exceeded")]
    TooManyGateways,
    
    #[msg("Invalid cross-chain message - validation pipeline failed")]
    InvalidCrossChainMessage,
    
    #[msg("Invalid payload - payload validation failed")]
    InvalidPayload,
    
    // Cross-chain message format errors
    #[msg("Invalid message type - must be 1-4")]
    InvalidMessageType,
    
    #[msg("Invalid token ID - must be greater than 0")]
    InvalidTokenId,
    
    #[msg("Invalid metadata URI - cannot be empty")]
    InvalidMetadataUri,
    
    #[msg("Invalid NFT name - cannot be empty")]
    InvalidNftName,
    
    #[msg("Invalid NFT symbol - cannot be empty")]
    InvalidNftSymbol,
    
    #[msg("Message too old - timestamp exceeds maximum age")]
    MessageTooOld,
    
    #[msg("Message too large - exceeds maximum size limit")]
    MessageTooLarge,
    
    #[msg("Message too small - below minimum size limit")]
    MessageTooSmall,
    
    #[msg("Serialization error - failed to serialize message")]
    SerializationError,
    
    #[msg("Deserialization error - failed to deserialize message")]
    DeserializationError,
}
