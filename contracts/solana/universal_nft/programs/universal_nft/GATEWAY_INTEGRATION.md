# ZetaChain Gateway Integration Guide

## Overview

This document describes the integration of the Universal NFT program with the ZetaChain Gateway system for cross-chain NFT transfers. The integration provides secure, validated cross-chain messaging with comprehensive replay protection and error handling.

## Architecture

### Core Components

1. **Gateway Validation Module** (`gateway_validation.rs`)

   - Uses `sysvar::instructions` to validate Gateway caller
   - Provides enhanced security for cross-chain entrypoints
   - Includes comprehensive logging and monitoring

2. **Gateway Constants** (`constants/gateway.rs`)

   - ZetaChain Gateway program ID and configuration
   - Instruction discriminators for Gateway operations
   - Fee calculation and message size limits

3. **Replay Protection** (`state/replay_protection.rs`)

   - Uses Gateway message IDs for uniqueness
   - Includes metadata for enhanced tracking
   - Provides time-based replay detection

4. **Error Handling** (`utils/gateway_error_handling.rs`)
   - Gateway-specific error codes and messages
   - Comprehensive logging and debugging
   - Operation result tracking

## Integration Points

### 1. Incoming NFT Transfers (Mint)

```rust
// Account structure includes InstructionsSysvar for Gateway validation
#[derive(Accounts)]
#[instruction(chain_id: u8, gateway_message_id: [u8; 32])]
pub struct MintNft<'info> {
    // ... other accounts ...

    /// The instructions sysvar for Gateway validation
    pub instructions_sysvar: Account<'info, InstructionsSysvar>,

    /// The replay protection account (PDA) for Gateway message ID
    #[account(
        init_if_needed,
        payer = payer,
        space = ReplayProtection::LEN,
        seeds = [GATEWAY_REPLAY_SEED, &chain_id.to_le_bytes(), &gateway_message_id],
        bump,
    )]
    pub replay_protection: Account<'info, ReplayProtection>,
}
```

**Validation Flow:**

1. Validate Gateway caller using `sysvar::instructions`
2. Run cross-chain message validation pipeline
3. Check replay protection using Gateway message ID
4. Mint NFT and update replay protection

### 2. Outgoing NFT Transfers (Burn)

```rust
// Uses real Gateway CPI calls instead of placeholders
fn send_cross_chain_message(
    gateway_program: &AccountInfo,
    gateway_state: &AccountInfo,
    gateway_custody: &AccountInfo,
    tss_account: &AccountInfo,
    owner_zeta_account: &AccountInfo,
    destination_chain: u8,
    payload: &[u8],
) -> Result<()> {
    // Validate Gateway program ID
    require!(
        gateway_program.key() == ZETA_CHAIN_GATEWAY_PROGRAM_ID_PUBKEY,
        UniversalNftError::UnauthorizedGateway
    );

    // Create instruction data for Gateway's deposit_and_call
    let mut instruction_data = Vec::new();
    instruction_data.extend_from_slice(&GATEWAY_DEPOSIT_AND_CALL_DISCRIMINATOR);
    // ... serialize other parameters ...

    // Execute CPI call to Gateway
    invoke_signed(&instruction, accounts.as_slice(), &[])?;

    Ok(())
}
```

**CPI Flow:**

1. Validate Gateway program ID
2. Prepare instruction data according to Gateway interface
3. Execute `deposit_and_call` CPI call
4. Handle success/failure with comprehensive logging

## Security Features

### 1. Gateway Caller Validation

All cross-chain entrypoints validate that the caller is the ZetaChain Gateway using `sysvar::instructions`:

```rust
let gateway_validation = validate_gateway_caller(
    &ctx.accounts.instructions_sysvar,
    0, // Current instruction index
)?;
```

### 2. Replay Protection

Uses Gateway message IDs for unique identification:

```rust
// PDA seeds: [GATEWAY_REPLAY_SEED, chain_id, gateway_message_id]
seeds = [GATEWAY_REPLAY_SEED, &chain_id.to_le_bytes(), &gateway_message_id]
```

### 3. Message Validation

Comprehensive validation pipeline including:

- Chain ID verification
- Payload size validation
- Signature validation
- Gateway authorization

## Error Handling

### Gateway Error Codes

- `1001`: Unauthorized Gateway
- `1002`: Gateway Not Active
- `1003`: Invalid Gateway Data
- `1004`: Gateway Call Failed
- `1005`: Message Already Processed
- `1006`: Invalid Cross-Chain Message

### Error Handling Pattern

```rust
use crate::utils::gateway_error_handling::*;

// Handle Gateway errors with detailed logging
let result = handle_gateway_error(
    GatewayOperationType::IncomingMessage,
    &UniversalNftError::UnauthorizedGateway,
    "mint_nft_handler",
);

// Log successful operations
let success_result = log_gateway_success(
    GatewayOperationType::IncomingMessage,
    "mint_nft_handler",
    "NFT minted successfully",
);
```

## Configuration

### Gateway Constants

```rust
// ZetaChain Gateway Program ID
pub const ZETA_CHAIN_GATEWAY_PROGRAM_ID: &str = "ZetaChainGateway111111111111111111111111111111111";

// Default gas limit and fees
pub const DEFAULT_GAS_LIMIT: u64 = 100_000;
pub const DEFAULT_ZETA_FEE_AMOUNT: u64 = 1_000_000; // 0.001 ZETA
```

### Program State

The `ProgramState` includes Gateway configuration:

```rust
pub struct ProgramState {
    // ... existing fields ...

    /// List of authorized gateways
    pub authorized_gateways: Vec<AuthorizedGateway>,

    /// ZetaChain Gateway program ID
    pub zeta_chain_gateway_program_id: Pubkey,

    /// Whether Gateway integration is active
    pub gateway_integration_active: bool,

    /// Timestamp of last Gateway update
    pub gateway_last_updated: i64,
}
```

## Usage Examples

### 1. Mint NFT from Cross-Chain Message

```typescript
// Client-side call
const mintParams = {
  source_chain_id: 2, // Base Sepolia
  destination_chain_id: 1, // Solana
  gateway_message_id: new Uint8Array(32), // From Gateway
  payload: metadataPayload,
  signatures: validatorSignatures,
  validators: validatorPubkeys,
  timestamp: Date.now() / 1000,
  metadata: {
    source_chain_name: 'Base Sepolia',
    destination_chain_name: 'Solana',
    token_id: 123,
    recipient_address: recipientBytes,
    context: 'Cross-chain NFT transfer',
  },
};

await program.methods
  .mintNftHandler(mintParams)
  .accounts({
    payer: wallet.publicKey,
    mint: mintKeypair.publicKey,
    tokenAccount: tokenAccount,
    programState: programState,
    replayProtection: replayProtectionPda,
    instructionsSysvar: instructionsSysvar,
    systemProgram: SystemProgram.programId,
    tokenProgram: TOKEN_PROGRAM_ID,
    associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
  })
  .rpc();
```

### 2. Burn NFT and Send Cross-Chain

```typescript
const burnParams = {
  destination_chain: 2, // Base Sepolia
  destination_address: recipientBytes,
  metadata_uri: 'ipfs://...',
};

await program.methods
  .burnNftAndTransfer(burnParams)
  .accounts({
    owner: wallet.publicKey,
    mint: mintKeypair.publicKey,
    tokenAccount: tokenAccount,
    // ... other required accounts ...
    gatewayProgram: ZETA_CHAIN_GATEWAY_PROGRAM_ID,
    gatewayState: gatewayState,
    gatewayCustody: gatewayCustody,
    tssAccount: tssAccount,
    ownerZetaAccount: ownerZetaAccount,
  })
  .rpc();
```

## Testing

### Gateway Validation Tests

```rust
#[test]
fn test_gateway_caller_validation() {
    // Test with valid Gateway caller
    // Test with invalid caller
    // Test with missing instructions sysvar
}

#[test]
fn test_replay_protection() {
    // Test first message processing
    // Test duplicate message rejection
    // Test Gateway message ID uniqueness
}
```

### Integration Tests

```rust
#[test]
fn test_cross_chain_nft_flow() {
    // Test complete flow: mint → burn → cross-chain transfer
    // Verify Gateway integration at each step
    // Check replay protection and error handling
}
```

## Monitoring and Debugging

### Logging

The integration provides comprehensive logging:

```
🔍 Validating Gateway caller using sysvar::instructions...
✅ Gateway caller validation successful!
   Caller Program ID: ZetaChainGateway111111111111111111111111111111111
   Validated at: 1703123456

🔒 Checking Gateway replay protection...
✅ Gateway replay protection updated successfully!
   Chain ID: 00000000000000000000000000000002
   Gateway Message ID: abc123...
   Processed at: 1703123456
```

### Statistics

Track Gateway operation performance:

```rust
let stats = get_gateway_operation_stats();
msg!("Gateway Operations: {}/{} successful",
     stats.successful_operations,
     stats.total_operations);
```

## Migration Guide

### From Legacy to Gateway

1. **Update Account Structures**: Add `InstructionsSysvar` for Gateway validation
2. **Replace Message Hashes**: Use Gateway message IDs instead of custom hashes
3. **Update PDA Seeds**: Change from `REPLAY_SEED` to `GATEWAY_REPLAY_SEED`
4. **Add Gateway Validation**: Integrate `validate_gateway_caller()` calls
5. **Update Error Handling**: Use Gateway-specific error codes and messages

### Backward Compatibility

Legacy handlers are maintained for backward compatibility:

```rust
pub fn mint_nft_legacy_handler(ctx: Context<MintNftLegacy>, params: CrossChainMintParamsLegacy) -> Result<()> {
    msg!("⚠️  Using legacy mint NFT handler - consider upgrading to Gateway-based version");
    // ... legacy implementation ...
}
```

## Troubleshooting

### Common Issues

1. **Gateway Validation Fails**

   - Ensure `InstructionsSysvar` is included in account list
   - Verify Gateway program ID matches expected value
   - Check that instruction is called by Gateway

2. **Replay Protection Errors**

   - Verify Gateway message ID uniqueness
   - Check PDA seed derivation
   - Ensure message hasn't been processed before

3. **CPI Call Failures**
   - Validate Gateway program ID
   - Check account structure matches Gateway requirements
   - Verify instruction data format

### Debug Commands

```bash
# Check Gateway integration status
solana program show ZetaChainGateway111111111111111111111111111111111

# Verify replay protection accounts
solana account <replay_protection_pda>

# Monitor program logs
solana logs <program_id>
```

## Future Enhancements

1. **Dynamic Gateway Management**: Add/remove authorized gateways
2. **Enhanced Metadata**: Support for complex cross-chain transfer metadata
3. **Performance Optimization**: Batch Gateway operations
4. **Advanced Security**: Multi-signature Gateway validation
5. **Monitoring**: Real-time Gateway operation metrics

## Support

For issues related to Gateway integration:

1. Check program logs for detailed error messages
2. Verify Gateway program deployment and configuration
3. Review replay protection account states
4. Consult ZetaChain Gateway documentation
5. Open issues with detailed error context and logs

