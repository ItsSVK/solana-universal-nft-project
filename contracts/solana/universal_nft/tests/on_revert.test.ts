import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { UniversalNft } from '../target/types/universal_nft';
import { PublicKey, SystemProgram, Keypair } from '@solana/web3.js';
import { expect } from 'chai';

anchor.setProvider(anchor.AnchorProvider.env());
const program = anchor.workspace.universalNft as Program<UniversalNft>;

describe('on_revert Handler', () => {
  it('should successfully handle on_revert when called by authorized Gateway', async () => {
    // Create test data
    const sender = new Array(32).fill(1); // number[] for Anchor array[32] u8

    const message = Buffer.from('Test message data for failed transfer');
    const reason = Buffer.from('Transfer failed due to insufficient funds');

    // Create a mock Gateway program key (placeholder)
    const mockGatewayProgram = new PublicKey([
      1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
      1, 1, 1, 1, 1, 1, 1,
    ]);

    // Call on_revert with valid parameters
    await program.methods
      .onRevert(sender, message, reason)
      .accounts({
        payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
        gatewayProgram: mockGatewayProgram,
      })
      .rpc();

    // The function should complete successfully without throwing errors
    // The logs will be visible in the transaction logs
  });

  it('should reject on_revert when called by unauthorized program', async () => {
    // Create test data
    const sender = new Array(32).fill(2); // number[] for Anchor array[32] u8

    const message = Buffer.from('Another test message');
    const reason = Buffer.from('Different failure reason');

    // Create an unauthorized program key (different from expected Gateway)
    const unauthorizedProgram = new PublicKey([
      2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
      2, 2, 2, 2, 2, 2, 2,
    ]);

    try {
      await program.methods
        .onRevert(sender, message, reason)
        .accounts({
          payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
          gatewayProgram: unauthorizedProgram,
        })
        .rpc();

      // If we reach here, the test should fail
      expect.fail('on_revert should have failed with unauthorized gateway');
    } catch (error) {
      // Expected to fail with UnauthorizedGateway error
      expect(error.message).to.include('UnauthorizedGateway');
    }
  });

  it('should handle on_revert with empty message and reason', async () => {
    // Create test data with empty vectors
    const sender = new Array(32).fill(3);

    const message = Buffer.from([]); // Empty message
    const reason = Buffer.from([]); // Empty reason

    // Create a mock Gateway program key
    const mockGatewayProgram = new PublicKey([
      1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
      1, 1, 1, 1, 1, 1, 1,
    ]);

    // Call on_revert with empty parameters
    await program.methods
      .onRevert(sender, message, reason)
      .accounts({
        payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
        gatewayProgram: mockGatewayProgram,
      })
      .rpc();

    // Should complete successfully even with empty parameters
  });

  it('should handle on_revert with large message and reason', async () => {
    // Create test data with large vectors (but within reasonable limits)
    const sender = new Array(32).fill(4);

    const message = Buffer.alloc(200, 'A'); // Large message (reduced size)
    const reason = Buffer.alloc(100, 'B'); // Large reason (reduced size)

    // Create a mock Gateway program key
    const mockGatewayProgram = new PublicKey([
      1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
      1, 1, 1, 1, 1, 1, 1,
    ]);

    // Call on_revert with large parameters
    await program.methods
      .onRevert(sender, message, reason)
      .accounts({
        payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
        gatewayProgram: mockGatewayProgram,
      })
      .rpc();

    // Should complete successfully with large parameters
  });

  it('should handle on_revert with UTF-8 encoded reason', async () => {
    // Create test data with UTF-8 encoded reason
    const sender = new Array(32).fill(5);

    const message = Buffer.from('Transfer failed');
    const reason = Buffer.from(
      '🚫 Transfer failed due to network congestion 🚫',
      'utf8'
    );

    // Create a mock Gateway program key
    const mockGatewayProgram = new PublicKey([
      1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
      1, 1, 1, 1, 1, 1, 1,
    ]);

    // Call on_revert with UTF-8 encoded reason
    await program.methods
      .onRevert(sender, message, reason)
      .accounts({
        payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
        gatewayProgram: mockGatewayProgram,
      })
      .rpc();

    // Should complete successfully with UTF-8 encoded reason
  });
});
