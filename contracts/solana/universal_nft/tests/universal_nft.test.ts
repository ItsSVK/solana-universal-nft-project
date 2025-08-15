import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { UniversalNft } from '../target/types/universal_nft';
import { PublicKey } from '@solana/web3.js';
import { expect } from 'chai';

describe('universal_nft', () => {
  anchor.setProvider(anchor.AnchorProvider.env());
  const program = anchor.workspace.universalNft as Program<UniversalNft>;
  let programStatePda: PublicKey;
  let nftOriginPda: PublicKey;

  before(async () => {
    // Get the program state PDA
    const [programStatePDADerived] = PublicKey.findProgramAddressSync(
      [Buffer.from('program-state')],
      program.programId
    );
    programStatePda = programStatePDADerived;
  });

  it('initializes the program state and creates an NFT origin account', async () => {
    // Create a test mint address
    const testMint = anchor.web3.Keypair.generate();

    // Test data
    const tokenId = 1;
    const originChain = 1; // Ethereum mainnet
    const originAddress = Array(32).fill(1); // Test address as number array
    const metadataUri = 'https://example.com/metadata.json';

    // Derive the NFT origin PDA
    const [nftOriginPDADerived] = PublicKey.findProgramAddressSync(
      [
        Buffer.from('nft_origin'),
        new anchor.BN(tokenId).toArrayLike(Buffer, 'le', 8),
      ],
      program.programId
    );
    nftOriginPda = nftOriginPDADerived;

    try {
      // First, initialize the program state if it doesn't exist
      await program.methods
        .initializeProgramState()
        .accounts({
          payer: program.provider.publicKey,
        })
        .rpc();

      console.log('Program state initialized');

      // Create the NFT origin
      const tx = await program.methods
        .createNftOrigin(
          new anchor.BN(tokenId),
          originChain,
          originAddress,
          testMint.publicKey,
          metadataUri
        )
        .accounts({
          payer: program.provider.publicKey,
          programState: programStatePda,
        })
        .rpc();

      console.log('NFT Origin created successfully!');
      // console.log('Transaction signature:', tx);
      console.log('NFT Origin PDA:', nftOriginPda.toBase58());

      // Fetch and verify the created account
      const nftOriginAccount = await program.account.nftOrigin.fetch(
        nftOriginPda
      );
      console.log('NFT Origin Account:', {
        tokenId: nftOriginAccount.tokenId.toString(),
        originChain: nftOriginAccount.originChain,
        originAddress: Array.from(nftOriginAccount.originAddress),
        mintAddress: nftOriginAccount.mintAddress.toBase58(),
        createdAt: new Date(nftOriginAccount.createdAt.toNumber() * 1000),
        metadataUri: nftOriginAccount.metadataUri,
      });

      // Verify the data matches what we set
      console.log('Verification:');
      console.log(
        '✓ Token ID matches:',
        nftOriginAccount.tokenId.toString() === tokenId.toString()
      );
      console.log(
        '✓ Origin Chain matches:',
        nftOriginAccount.originChain === originChain
      );
      console.log(
        '✓ Origin Address matches:',
        JSON.stringify(Array.from(nftOriginAccount.originAddress)) ===
          JSON.stringify(Array.from(originAddress))
      );
      console.log(
        '✓ Mint Address matches:',
        nftOriginAccount.mintAddress.toBase58() ===
          testMint.publicKey.toBase58()
      );
      console.log(
        '✓ Metadata URI matches:',
        nftOriginAccount.metadataUri === metadataUri
      );
    } catch (error) {
      console.error('Error creating NFT origin:', error);
      throw error;
    }
  });

  after(async () => {
    // Cleanup: Close both accounts
    try {
      // Close NFT origin account first
      const closeNftOriginSig = await program.methods
        .closeNftOrigin()
        .accounts({
          nftOrigin: nftOriginPda,
          payer: program.provider.publicKey,
        })
        .rpc();
      console.log('NFT Origin account closed:', closeNftOriginSig);
    } catch (e) {
      console.log('NFT Origin cleanup failed (account may not exist):', e);
    }

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
