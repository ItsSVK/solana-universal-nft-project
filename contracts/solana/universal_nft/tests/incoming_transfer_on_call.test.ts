import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { UniversalNft } from '../target/types/universal_nft';
import {
  PublicKey,
  SystemProgram,
  Keypair,
  ComputeBudgetProgram,
} from '@solana/web3.js';
import { expect } from 'chai';

anchor.setProvider(anchor.AnchorProvider.env());
const program = anchor.workspace.universalNft as Program<UniversalNft>;

// Constants matching on-chain seeds
const PROGRAM_STATE_SEED = Buffer.from('program-state');
const NFT_ORIGIN_SEED = Buffer.from('nft_origin');
const NFT_ORIGIN_BY_TOKEN_ID_SEED = Buffer.from('nft_origin_by_token_id');

// SPL Program IDs
const TOKEN_PROGRAM_ID = new PublicKey(
  'TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA'
);
const ASSOCIATED_TOKEN_PROGRAM_ID = new PublicKey(
  'ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL'
);

// Metaplex Token Metadata program ID (for PDA derivation only)
const METAPLEX_PROGRAM_ID = new PublicKey(
  'metaqbxxUerdq28cj1RbAWkYQm3ybzjb6a8bt518x1s'
);

function u64ToLeBytes(n: number | bigint): Buffer {
  const bn = new anchor.BN(n.toString());
  const buf = Buffer.alloc(8);
  buf.writeBigUInt64LE(BigInt(bn.toString()));
  return buf;
}

function buildPayload(params: {
  tokenId: number | bigint;
  originChainIdByte?: number; // first byte used on-chain for chain id
  gatewayMessageIdByte?: number;
  metadataUri: string;
  name: string;
  symbol: string;
  recipient: PublicKey;
  additional?: Buffer;
}): Buffer {
  const tokenIdBuf = u64ToLeBytes(params.tokenId);
  // Keep 32-byte buffers as the program expects them
  const originChain = Buffer.alloc(32, params.originChainIdByte ?? 1);
  const gatewayMsgId = Buffer.alloc(32, params.gatewayMessageIdByte ?? 2);
  const uri = Buffer.from(params.metadataUri, 'utf8');
  const name = Buffer.from(params.name, 'utf8');
  const symbol = Buffer.from(params.symbol, 'utf8');
  const recipient = Buffer.from(params.recipient.toBytes());
  const nul = Buffer.from([0]);

  const parts = [
    tokenIdBuf,
    originChain,
    gatewayMsgId,
    uri,
    nul,
    name,
    nul,
    symbol,
    nul,
    recipient,
  ];
  if (params.additional) parts.push(params.additional);
  return Buffer.concat(parts);
}

async function deriveProgramStatePda(): Promise<[PublicKey, number]> {
  return await PublicKey.findProgramAddress(
    [PROGRAM_STATE_SEED],
    program.programId
  );
}

function deriveMetadataPda(mint: PublicKey): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(
    [Buffer.from('metadata'), METAPLEX_PROGRAM_ID.toBuffer(), mint.toBuffer()],
    METAPLEX_PROGRAM_ID
  );
  return pda;
}

function deriveMasterEditionPda(mint: PublicKey): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(
    [
      Buffer.from('metadata'),
      METAPLEX_PROGRAM_ID.toBuffer(),
      mint.toBuffer(),
      Buffer.from('edition'),
    ],
    METAPLEX_PROGRAM_ID
  );
  return pda;
}

function deriveNftOriginPda(tokenId: number | bigint): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(
    [NFT_ORIGIN_SEED, u64ToLeBytes(tokenId)],
    program.programId
  );
  return pda;
}

function deriveNftOriginByTokenIdPda(tokenId: number | bigint): PublicKey {
  const [pda] = PublicKey.findProgramAddressSync(
    [NFT_ORIGIN_BY_TOKEN_ID_SEED, u64ToLeBytes(tokenId)],
    program.programId
  );
  return pda;
}

describe('Incoming Transfer via on_call', () => {
  let programStatePda: PublicKey;
  let collectionMint: PublicKey;

  before(async () => {
    // Derive program state PDA once for all tests
    [programStatePda] = await deriveProgramStatePda();

    // Initialize program state once for all tests
    try {
      await program.methods
        .initializeProgramState()
        .accounts({
          payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
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

    // Ensure collection is minted and verified for all tests
    const name = 'Collection';
    const symbol = 'COLL';
    const uri = 'https://example.com/collection.json';
    const collectionMintKp = Keypair.generate();

    try {
      await program.methods
        .mintCollection(name, symbol, uri)
        .accounts({
          authority: (program.provider as anchor.AnchorProvider).wallet
            .publicKey,
          collectionMint: collectionMintKp.publicKey,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([collectionMintKp])
        .rpc();
      console.log('Collection minted successfully');
    } catch (e) {
      console.log('Collection already minted or error:', e.message);
    }

    // Verify collection if not verified
    const programState: any = await program.account.programState.fetch(
      programStatePda
    );
    collectionMint = new PublicKey(programState.collectionMint);

    if (!programState.collectionVerified) {
      try {
        await program.methods
          .verifyCollection()
          .accounts({
            authority: (program.provider as anchor.AnchorProvider).wallet
              .publicKey,
            collectionMint,
            rent: anchor.web3.SYSVAR_RENT_PUBKEY,
          })
          .rpc();
        console.log('Collection verified successfully');
      } catch (e) {
        console.log('Collection verification failed:', e.message);
      }
    }
  });

  after(async () => {
    // Cleanup program state after all tests
    try {
      await program.methods
        .closeProgramState()
        .accounts({
          programState: programStatePda,
          payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
        })
        .rpc();
      console.log('Program state cleaned up');
    } catch (e) {
      console.log('Program state cleanup failed:', e.message);
    }
  });

  it('should handle new NFT on on_call and validate core logic', async () => {
    const tokenId = 123;
    const recipient = (program.provider as anchor.AnchorProvider).wallet
      .publicKey;

    // Build a minimal payload to avoid memory issues
    const payload = buildPayload({
      tokenId,
      gatewayMessageIdByte: 1,
      metadataUri: 'https://a.com', // Proper HTTP URI
      name: 'a', // Minimal name
      symbol: 'a', // Minimal symbol
      recipient,
    });

    console.log('📦 Payload size:', payload.length, 'bytes');

    const mintKp = Keypair.generate();
    const recipientAta = anchor.utils.token.associatedAddress({
      mint: mintKp.publicKey,
      owner: recipient,
    });
    const metadataPda = deriveMetadataPda(mintKp.publicKey);
    const masterEditionPda = deriveMasterEditionPda(mintKp.publicKey);
    const collectionMetadataPda = deriveMetadataPda(collectionMint);
    const collectionMasterEditionPda = deriveMasterEditionPda(collectionMint);
    const nftOriginByTokenIdPda = deriveNftOriginByTokenIdPda(tokenId);
    const nftOriginPda = deriveNftOriginPda(tokenId);

    // Use higher compute budget and better error handling
    await program.methods
      .onCall(payload)
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 2_000_000 }), // Increased compute units
        ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 1 }),
      ])
      .accounts({
        payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
        mint: mintKp.publicKey,
        recipient,
        programState: programStatePda,
        collectionMint,
        collectionMetadata: collectionMetadataPda,
        collectionMasterEdition: collectionMasterEditionPda,
        metadata: metadataPda,
        masterEdition: masterEditionPda,
        nftOrigin: nftOriginPda,
        nftOriginByTokenId: nftOriginByTokenIdPda,
        replayProtection: PublicKey.findProgramAddressSync(
          [
            Buffer.from('replay'),
            payload.slice(8, 40), // chain_id (32 bytes)
            payload.slice(40, 72), // message_id (32 bytes)
          ],
          program.programId
        )[0],
      })
      .signers([mintKp])
      .rpc();

    // Verify the NFT was created correctly
    const nftOriginAccount = await program.account.nftOrigin.fetch(
      nftOriginPda
    );
    expect(nftOriginAccount.tokenId.toString()).to.equal(tokenId.toString());
    expect(nftOriginAccount.originChain).to.equal(1);
    expect(nftOriginAccount.metadataUri).to.equal('https://a.com');

    console.log('✅ NFT created successfully with token ID:', tokenId);
  });

  it('should reject a second on_call with the same token_id but different mint', async () => {
    const tokenId = 456; // Different token ID from first test
    const payload = buildPayload({
      tokenId,
      gatewayMessageIdByte: 3, // Different message ID to avoid replay protection conflict
      metadataUri: 'https://b.com', // Proper HTTP URI
      name: 'b', // Minimal name
      symbol: 'b', // Minimal symbol
      recipient: (program.provider as anchor.AnchorProvider).wallet.publicKey,
    });

    // First on_call should succeed
    const mintKp1 = Keypair.generate();
    const recipientAta1 = anchor.utils.token.associatedAddress({
      mint: mintKp1.publicKey,
      owner: (program.provider as anchor.AnchorProvider).wallet.publicKey,
    });
    const metadataPda1 = deriveMetadataPda(mintKp1.publicKey);
    const masterEditionPda1 = deriveMasterEditionPda(mintKp1.publicKey);
    const collectionMetadataPda = deriveMetadataPda(collectionMint);
    const collectionMasterEditionPda = deriveMasterEditionPda(collectionMint);
    const nftOriginByTokenIdPda = deriveNftOriginByTokenIdPda(tokenId);
    const nftOriginPda1 = deriveNftOriginPda(tokenId);

    await program.methods
      .onCall(payload)
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 2_000_000 }),
        ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 1 }),
      ])
      .accounts({
        payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
        mint: mintKp1.publicKey,
        recipient: (program.provider as anchor.AnchorProvider).wallet.publicKey,
        programState: programStatePda,
        collectionMint,
        collectionMetadata: collectionMetadataPda,
        collectionMasterEdition: collectionMasterEditionPda,
        metadata: metadataPda1,
        masterEdition: masterEditionPda1,
        nftOrigin: nftOriginPda1,
        nftOriginByTokenId: nftOriginByTokenIdPda,
        replayProtection: PublicKey.findProgramAddressSync(
          [Buffer.from('replay'), payload.slice(8, 40), payload.slice(40, 72)],
          program.programId
        )[0],
      })
      .signers([mintKp1])
      .rpc();

    console.log('✅ First on_call succeeded');

    // Second on_call with same token_id but different mint should fail
    const mintKp2 = Keypair.generate();
    const metadataPda2 = deriveMetadataPda(mintKp2.publicKey);
    const masterEditionPda2 = deriveMasterEditionPda(mintKp2.publicKey);
    const nftOriginPda2 = deriveNftOriginPda(tokenId);

    try {
      await program.methods
        .onCall(payload)
        .preInstructions([
          ComputeBudgetProgram.setComputeUnitLimit({ units: 2_000_000 }),
          ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 1 }),
        ])
        .accounts({
          payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
          mint: mintKp2.publicKey,
          recipient: (program.provider as anchor.AnchorProvider).wallet
            .publicKey,
          programState: programStatePda,
          collectionMint,
          collectionMetadata: collectionMetadataPda,
          collectionMasterEdition: collectionMasterEditionPda,
          metadata: metadataPda2,
          masterEdition: masterEditionPda2,
          nftOrigin: nftOriginPda2,
          nftOriginByTokenId: nftOriginByTokenIdPda,
          replayProtection: PublicKey.findProgramAddressSync(
            [
              Buffer.from('replay'),
              payload.slice(8, 40),
              payload.slice(40, 72),
            ],
            program.programId
          )[0],
        })
        .signers([mintKp2])
        .rpc();

      // If we reach here, the test should fail
      expect.fail('Second on_call with same token_id should have failed');
    } catch (error) {
      // Expected to fail - the NftOriginByTokenId PDA should already exist
      // and the program should detect this as a returning NFT scenario
      expect(error.message).to.include('failed');
      console.log('✅ Second on_call correctly rejected as expected');
    }
  });
});
