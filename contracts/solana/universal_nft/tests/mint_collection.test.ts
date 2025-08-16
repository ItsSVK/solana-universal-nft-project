import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { UniversalNft } from '../target/types/universal_nft';
import { PublicKey, Keypair } from '@solana/web3.js';
import { expect } from 'chai';

describe('Mint Collection NFT', () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.universalNft as Program<UniversalNft>;
  let programStatePda: PublicKey;
  it('should mint Collection NFT successfully', async () => {
    console.log('Testing Collection NFT minting...');

    // Test data
    const collectionName = 'Universal NFT Collection';
    const collectionSymbol = 'UNFT';
    const collectionUri = 'https://example.com/collection-metadata.json';

    // Get the program state PDA
    const [programStatePDADerived] = PublicKey.findProgramAddressSync(
      [Buffer.from('program-state')],
      program.programId
    );
    programStatePda = programStatePDADerived;
    // Create a new mint for the collection
    const collectionMint = Keypair.generate();

    // Get the authority's token account PDA
    // const [authorityTokenAccount] = PublicKey.findProgramAddressSync(
    //   [
    //     anchor.utils.bytes.bs58.decode(
    //       anchor.utils.token.ASSOCIATED_PROGRAM_ID.toString()
    //     ),
    //     program.provider.publicKey.toBuffer(),
    //     collectionMint.publicKey.toBuffer(),
    //   ],
    //   anchor.utils.token.ASSOCIATED_PROGRAM_ID
    // );

    const authorityTokenAccount = anchor.utils.token.associatedAddress({
      mint: collectionMint.publicKey,
      owner: program.provider.publicKey,
    });

    console.log('Program State PDA:', programStatePda.toString());
    console.log('Collection Mint:', collectionMint.publicKey.toString());
    console.log('Authority Token Account:', authorityTokenAccount.toString());
    console.log('Authority:', program.provider.publicKey.toString());

    try {
      // Initialize program state first if needed
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

      // Mint the Collection NFT
      const tx = await program.methods
        .mintCollection(collectionName, collectionSymbol, collectionUri)
        .accounts({
          authority: program.provider.publicKey,
          collectionMint: collectionMint.publicKey,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([collectionMint])
        .rpc();

      console.log('Collection NFT minted successfully!');
      console.log('Transaction signature:', tx);
      console.log(
        'Collection Mint Address:',
        collectionMint.publicKey.toString()
      );

      // Fetch the program state to verify collection mint was updated
      const programStateAccount = await program.account.programState.fetch(
        programStatePda
      );

      // Verify the collection mint address was updated
      expect(programStateAccount.collectionMint.toString()).to.equal(
        collectionMint.publicKey.toString()
      );
      expect(programStateAccount.collectionVerified).to.be.false; // Should still be false

      console.log('✅ Collection mint address updated in program state');
      console.log('✅ Collection verification status is false (as expected)');

      // Verify the token account has 1 token
      const tokenAccountInfo =
        await program.provider.connection.getTokenAccountBalance(
          authorityTokenAccount
        );

      expect(tokenAccountInfo.value.amount).to.equal('1');
      expect(tokenAccountInfo.value.uiAmount).to.equal(1);

      console.log('✅ Token account verified - 1 token minted to authority');
    } catch (error) {
      console.log('Error:', error.message);
      console.log('Error details:', error);
      throw error;
    }
  });

  it('should fail when trying to mint collection twice', async () => {
    console.log('Testing duplicate collection minting prevention...');

    // Test data
    const collectionName = 'Duplicate Collection';
    const collectionSymbol = 'DUP';
    const collectionUri = 'https://example.com/duplicate-metadata.json';

    // Get the program state PDA
    const [programStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from('program-state')],
      program.programId
    );

    // Create a new mint for the duplicate collection
    const duplicateMint = Keypair.generate();

    // Get the authority's token account PDA
    const [authorityTokenAccount] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.bs58.decode(
          anchor.utils.token.ASSOCIATED_PROGRAM_ID.toString()
        ),
        program.provider.publicKey.toBuffer(),
        duplicateMint.publicKey.toBuffer(),
      ],
      anchor.utils.token.ASSOCIATED_PROGRAM_ID
    );

    try {
      // Try to mint another collection (should fail)
      await program.methods
        .mintCollection(collectionName, collectionSymbol, collectionUri)
        .accounts({
          authority: program.provider.publicKey,
          collectionMint: duplicateMint.publicKey,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([duplicateMint])
        .rpc();

      // If we reach here, the test should fail
      expect.fail('Expected error for duplicate collection minting');
    } catch (error) {
      console.log('✅ Correctly prevented duplicate collection minting');
      console.log('Error message:', error.message);
      expect(error.message).to.contain('CollectionAlreadyMinted');
    }
  });

  it('should validate input parameters', async () => {
    console.log('Testing input parameter validation...');

    // Get the program state PDA
    const [programStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from('program-state')],
      program.programId
    );

    // Create a new mint for testing
    const testMint = Keypair.generate();

    // Get the authority's token account PDA
    const [authorityTokenAccount] = PublicKey.findProgramAddressSync(
      [
        anchor.utils.bytes.bs58.decode(
          anchor.utils.token.ASSOCIATED_PROGRAM_ID.toString()
        ),
        program.provider.publicKey.toBuffer(),
        testMint.publicKey.toBuffer(),
      ],
      anchor.utils.token.ASSOCIATED_PROGRAM_ID
    );

    // Test with empty name
    try {
      await program.methods
        .mintCollection('', 'TEST', 'https://example.com/test.json')
        .accounts({
          authority: program.provider.publicKey,
          collectionMint: testMint.publicKey,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([testMint])
        .rpc();

      expect.fail('Expected error for empty name');
    } catch (error) {
      console.log('✅ Correctly validated empty name');
      expect(error.message).to.contain('InvalidCollectionData');
    }
  });
  after(async () => {
    try {
      // Close program state account
      const closeProgramStateSig = await program.methods
        .closeProgramState()
        .accounts({
          programState: programStatePda,
          payer: program.provider.publicKey,
        })
        .rpc();
      console.log('Program state account closed:', closeProgramStateSig);
    } catch (e) {
      console.log('Program state cleanup failed (account may not exist):', e);
    }
  });
});
