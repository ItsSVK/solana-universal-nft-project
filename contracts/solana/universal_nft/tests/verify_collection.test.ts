import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { UniversalNft } from '../target/types/universal_nft';
import { PublicKey, Keypair } from '@solana/web3.js';
import { expect } from 'chai';

describe('Verify Collection NFT', () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.universalNft as Program<UniversalNft>;
  let programStatePda: PublicKey;
  let collectionMint: Keypair;
  let authorityTokenAccount: PublicKey;

  it('should verify collection NFT successfully', async () => {
    console.log('Testing Collection NFT verification...');

    // Get the program state PDA
    const [programStatePDADerived] = PublicKey.findProgramAddressSync(
      [Buffer.from('program-state')],
      program.programId
    );
    programStatePda = programStatePDADerived;

    // Create a new mint for the collection
    collectionMint = Keypair.generate();

    // Get the authority's token account PDA
    authorityTokenAccount = anchor.utils.token.associatedAddress({
      mint: collectionMint.publicKey,
      owner: program.provider.publicKey,
    });

    console.log('Program State PDA:', programStatePda.toString());
    console.log('Collection Mint:', collectionMint.publicKey.toString());
    console.log('Authority Token Account:', authorityTokenAccount.toString());
    console.log('Authority:', program.provider.publicKey.toString());

    try {
      // Step 1: Initialize program state first if needed
      try {
        await program.methods
          .initializeProgramState()
          .accounts({
            payer: program.provider.publicKey,
          })
          .rpc();
        console.log('Program state initialized');
      } catch (error) {
        if (error.message.includes('already in use')) {
          console.log('Program state already initialized');
        } else {
          throw error;
        }
      }

      // Step 2: Mint the Collection NFT first (or use existing if already minted)
      const collectionName = 'Universal NFT Collection';
      const collectionSymbol = 'UNFT';
      const collectionUri = 'https://example.com/collection-metadata.json';

      console.log('Minting Collection NFT...');
      try {
        const mintTx = await program.methods
          .mintCollection(collectionName, collectionSymbol, collectionUri)
          .accounts({
            authority: program.provider.publicKey,
            collectionMint: collectionMint.publicKey,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .signers([collectionMint])
          .rpc();

        console.log('Collection NFT minted successfully!');
        console.log('Mint transaction signature:', mintTx);
      } catch (error) {
        if (error.message.includes('CollectionAlreadyMinted')) {
          console.log(
            'Collection NFT already exists, proceeding with verification...'
          );

          // Get the existing collection mint from program state
          const programStateAccount = await program.account.programState.fetch(
            programStatePda
          );
          if (
            programStateAccount.collectionMint.toString() !==
            collectionMint.publicKey.toString()
          ) {
            // Update our collectionMint to use the existing one
            collectionMint = {
              publicKey: programStateAccount.collectionMint,
            } as any;
            // Update the authority token account for the existing collection
            authorityTokenAccount = anchor.utils.token.associatedAddress({
              mint: collectionMint.publicKey,
              owner: program.provider.publicKey,
            });
            console.log(
              'Updated to use existing collection mint:',
              collectionMint.publicKey.toString()
            );
            console.log(
              'Updated authority token account:',
              authorityTokenAccount.toString()
            );
          }
        } else {
          throw error;
        }
      }

      // Step 3: Verify the collection (or skip if already verified)
      console.log('Verifying Collection NFT...');

      // Check if collection is already verified
      const currentProgramState = await program.account.programState.fetch(
        programStatePda
      );
      if (currentProgramState.collectionVerified) {
        console.log(
          'Collection NFT already verified, skipping verification...'
        );
      } else {
        const verifyTx = await program.methods
          .verifyCollection()
          .accounts({
            authority: program.provider.publicKey,
            collectionMint: collectionMint.publicKey,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .rpc();

        console.log('Collection NFT verified successfully!');
        console.log('Verification transaction signature:', verifyTx);
      }

      // Step 4: Verify the program state was updated
      const programStateAccount = await program.account.programState.fetch(
        programStatePda
      );

      expect(programStateAccount.collectionMint.toString()).to.equal(
        collectionMint.publicKey.toString()
      );
      expect(programStateAccount.collectionVerified).to.be.true;

      console.log('✅ Collection verification status updated in program state');
      console.log(
        '✅ Collection mint address:',
        programStateAccount.collectionMint.toString()
      );
      console.log(
        '✅ Collection verified:',
        programStateAccount.collectionVerified
      );
    } catch (error) {
      console.log('Error:', error.message);
      console.log('Error details:', error);
      throw error;
    }
  });

  it('should fail when trying to verify collection twice', async () => {
    console.log('Testing duplicate collection verification prevention...');

    // Ensure we have the necessary variables from the first test
    if (!collectionMint || !programStatePda || !authorityTokenAccount) {
      throw new Error('Test setup failed - missing required variables');
    }

    try {
      // Try to verify the collection again (should fail)
      await program.methods
        .verifyCollection()
        .accounts({
          authority: program.provider.publicKey,
          collectionMint: collectionMint.publicKey,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc();

      // If we reach here, the test should fail
      expect.fail('Expected error for duplicate collection verification');
    } catch (error) {
      console.log('✅ Correctly prevented duplicate collection verification');
      console.log('Error message:', error.message);
      expect(error.message).to.contain('CollectionAlreadyMinted');
    }
  });

  it('should fail when trying to verify unminted collection', async () => {
    console.log('Testing verification of unminted collection...');

    // Create a new program state for testing
    const testProgramState = Keypair.generate();
    const testCollectionMint = Keypair.generate();
    const testAuthorityTokenAccount = anchor.utils.token.associatedAddress({
      mint: testCollectionMint.publicKey,
      owner: program.provider.publicKey,
    });

    try {
      // Try to verify a collection that hasn't been minted (should fail)
      await program.methods
        .verifyCollection()
        .accounts({
          authority: program.provider.publicKey,
          collectionMint: testCollectionMint.publicKey,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc();

      // If we reach here, the test should fail
      expect.fail('Expected error for unminted collection verification');
    } catch (error) {
      console.log('✅ Correctly prevented verification of unminted collection');
      console.log('Error message:', error.message);
      // This will likely fail due to account not found, but that's expected
      expect(error.message).to.contain('AccountNotInitialized');
    }
  });
});
