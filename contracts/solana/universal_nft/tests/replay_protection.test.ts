import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { UniversalNft } from '../target/types/universal_nft';
import { PublicKey } from '@solana/web3.js';
import { expect } from 'chai';

describe('replay_protection', () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.universalNft as Program<UniversalNft>;

  it('should mark a new message as processed', async () => {
    // Create test data with unique values using timestamp
    const timestamp = Date.now();
    const chainId = Array(32).fill(1); // Chain ID 1
    const messageHash = Array(32).fill(timestamp % 256); // Unique message hash based on timestamp

    // Derive the replay protection PDA
    const [replayProtectionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('replay'), Buffer.from(chainId), Buffer.from(messageHash)],
      program.programId
    );

    console.log('Replay Protection PDA:', replayProtectionPda.toString());
    console.log('Using timestamp:', timestamp);

    // Call the check_and_mark_message instruction
    const tx = await program.methods
      .checkAndMarkMessage(chainId, messageHash)
      .accounts({
        payer: program.provider.publicKey,
      })
      .rpc();

    console.log('Transaction signature:', tx);

    // Verify the account was created and contains the expected data
    const replayProtectionAccount =
      await program.account.replayProtection.fetch(replayProtectionPda);

    console.log('Replay Protection Account:');
    console.log('- Bump:', replayProtectionAccount.bump);
    console.log('- Processed at:', replayProtectionAccount.processedAt);
    console.log('- Chain ID:', replayProtectionAccount.chainId);
    console.log(
      '- Gateway Message ID:',
      replayProtectionAccount.gatewayMessageId
    );

    // Verify the data matches our input
    expect(replayProtectionAccount.chainId).to.deep.equal(chainId);
    expect(replayProtectionAccount.gatewayMessageId).to.deep.equal(messageHash);
    expect(replayProtectionAccount.processedAt.toNumber()).to.be.greaterThan(0);
  });

  it('should reject duplicate messages', async () => {
    // Create test data with unique values using timestamp
    const timestamp = Date.now();
    const chainId = Array(32).fill(3); // Chain ID 3
    const messageHash = Array(32).fill((timestamp + 1) % 256); // Unique message hash

    // First call should succeed
    const [replayProtectionPda] = PublicKey.findProgramAddressSync(
      [Buffer.from('replay'), Buffer.from(chainId), Buffer.from(messageHash)],
      program.programId
    );

    await program.methods
      .checkAndMarkMessage(chainId, messageHash)
      .accounts({
        payer: program.provider.publicKey,
      })
      .rpc();

    console.log('First message processed successfully');

    // Second call with the same data should fail
    try {
      await program.methods
        .checkAndMarkMessage(chainId, messageHash)
        .accounts({
          payer: program.provider.publicKey,
        })
        .rpc();

      // If we reach here, the test should fail
      expect(true).to.be.false;
    } catch (error) {
      console.log('Duplicate message correctly rejected:', error.message);
      expect(error.message).to.contain('MessageAlreadyProcessed');
    }
  });

  it('should allow different messages with same chain ID', async () => {
    const timestamp = Date.now();
    const chainId = Array(32).fill(5); // Same chain ID
    const messageHash1 = Array(32).fill((timestamp + 2) % 256); // Different message hash 1
    const messageHash2 = Array(32).fill((timestamp + 3) % 256); // Different message hash 2

    // First message
    const [replayProtectionPda1] = PublicKey.findProgramAddressSync(
      [Buffer.from('replay'), Buffer.from(chainId), Buffer.from(messageHash1)],
      program.programId
    );

    await program.methods
      .checkAndMarkMessage(chainId, messageHash1)
      .accounts({
        payer: program.provider.publicKey,
      })
      .rpc();

    console.log('First message processed');

    // Second message with different hash should succeed
    const [replayProtectionPda2] = PublicKey.findProgramAddressSync(
      [Buffer.from('replay'), Buffer.from(chainId), Buffer.from(messageHash2)],
      program.programId
    );

    await program.methods
      .checkAndMarkMessage(chainId, messageHash2)
      .accounts({
        payer: program.provider.publicKey,
      })
      .rpc();

    console.log('Second message processed successfully');

    // Verify both accounts exist and have different addresses
    expect(replayProtectionPda1.toString()).to.not.equal(
      replayProtectionPda2.toString()
    );
  });

  it('should allow same message hash with different chain IDs', async () => {
    const timestamp = Date.now();
    const chainId1 = Array(32).fill(8); // Different chain ID 1
    const chainId2 = Array(32).fill(9); // Different chain ID 2
    const messageHash = Array(32).fill((timestamp + 4) % 256); // Same message hash

    // First message
    const [replayProtectionPda1] = PublicKey.findProgramAddressSync(
      [Buffer.from('replay'), Buffer.from(chainId1), Buffer.from(messageHash)],
      program.programId
    );

    await program.methods
      .checkAndMarkMessage(chainId1, messageHash)
      .accounts({
        payer: program.provider.publicKey,
      })
      .rpc();

    console.log('First message processed');

    // Second message with different chain ID should succeed
    const [replayProtectionPda2] = PublicKey.findProgramAddressSync(
      [Buffer.from('replay'), Buffer.from(chainId2), Buffer.from(messageHash)],
      program.programId
    );

    await program.methods
      .checkAndMarkMessage(chainId2, messageHash)
      .accounts({
        payer: program.provider.publicKey,
      })
      .rpc();

    console.log('Second message processed successfully');

    // Verify both accounts exist and have different addresses
    expect(replayProtectionPda1.toString()).to.not.equal(
      replayProtectionPda2.toString()
    );
  });
});
