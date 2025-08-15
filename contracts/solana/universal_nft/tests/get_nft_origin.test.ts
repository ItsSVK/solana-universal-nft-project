import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { UniversalNft } from '../target/types/universal_nft';
import { PublicKey } from '@solana/web3.js';

describe('get_nft_origin', () => {
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

  it('creates and then fetches an NFT origin account', async () => {
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
      const createTx = await program.methods
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
      console.log('Create transaction signature:', createTx);
      console.log('NFT Origin PDA:', nftOriginPda.toBase58());

      // Now fetch the NFT origin using our new instruction
      const fetchTx = await program.methods
        .getNftOrigin(new anchor.BN(tokenId))
        .accounts({
          nftOrigin: nftOriginPda,
        })
        .rpc();

      console.log('NFT Origin fetched successfully!');
      console.log('Fetch transaction signature:', fetchTx);

      // Fetch and verify the account data directly
      const nftOriginAccount = await program.account.nftOrigin.fetch(
        nftOriginPda
      );
      console.log('NFT Origin Account Data:', {
        tokenId: nftOriginAccount.tokenId.toString(),
        originChain: nftOriginAccount.originChain,
        originAddress: Array.from(nftOriginAccount.originAddress),
        mintAddress: nftOriginAccount.mintAddress.toBase58(),
        createdAt: new Date(nftOriginAccount.createdAt.toNumber() * 1000),
        metadataUri: nftOriginAccount.metadataUri,
      });
    } catch (error) {
      console.error('Error in get_nft_origin test:', error);
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
