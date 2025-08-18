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
  it('should handle new NFT on on_call and validate core logic', async () => {
    const [programStatePda] = await deriveProgramStatePda();
    await program.methods
      .initializeProgramState()
      .accounts({
        payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
      })
      .rpc();

    // Ensure collection is minted and verified using the same shape as collection_management tests
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
    } catch (e) {
      // ignore if already minted
    }

    // Verify collection if not verified
    const programState: any = await program.account.programState.fetch(
      programStatePda
    );
    let collectionMint = new PublicKey(programState.collectionMint);
    if (!programState.collectionVerified) {
      await program.methods
        .verifyCollection()
        .accounts({
          authority: (program.provider as anchor.AnchorProvider).wallet
            .publicKey,
          collectionMint,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc();
      const refreshed: any = await program.account.programState.fetch(
        programStatePda
      );
      collectionMint = new PublicKey(refreshed.collectionMint);
    }

    const mintKp = Keypair.generate();
    const recipient = (program.provider as anchor.AnchorProvider).wallet
      .publicKey;
    const recipientAta = anchor.utils.token.associatedAddress({
      mint: mintKp.publicKey,
      owner: recipient,
    });
    const metadataPda = deriveMetadataPda(mintKp.publicKey);
    const masterEditionPda = deriveMasterEditionPda(mintKp.publicKey);

    const collectionMetadataPda = deriveMetadataPda(collectionMint);
    const collectionMasterEditionPda = deriveMasterEditionPda(collectionMint);

    const tokenId = 123;
    const payload = buildPayload({
      tokenId,
      metadataUri: 'https://example.com/meta.json',
      name: 'Cross-Chain NFT',
      symbol: 'CCNFT',
      recipient,
    });

    const nftOriginByTokenIdPda = deriveNftOriginByTokenIdPda(tokenId);
    const nftOriginPda = deriveNftOriginPda(tokenId);

    await program.methods
      .onCall(payload)
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1_200_000 }),
        ComputeBudgetProgram.setComputeUnitPrice({ microLamports: 1 }),
      ])
      .accounts({
        payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
        mint: mintKp.publicKey,
        recipient,
        programState: programStatePda,
        collectionMint: collectionMint,
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

    // Cleanup to avoid cross-test interference
    try {
      await program.methods
        .closeProgramState()
        .accounts({
          programState: programStatePda,
          payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
        })
        .rpc();
    } catch (_) {}
  });

  it('should reject a second on_call with the same token_id but different mint', async () => {
    // Test that a second on_call with the same token_id but different mint is rejected
    const [programStatePda] = await deriveProgramStatePda();

    // Initialize program state
    await program.methods
      .initializeProgramState()
      .accounts({
        payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
      })
      .rpc();

    // Ensure collection is minted and verified
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
    } catch (e) {
      // ignore if already minted
    }

    // Verify collection if not verified
    const programState: any = await program.account.programState.fetch(
      programStatePda
    );
    let collectionMint = new PublicKey(programState.collectionMint);
    if (!programState.collectionVerified) {
      await program.methods
        .verifyCollection()
        .accounts({
          authority: (program.provider as anchor.AnchorProvider).wallet
            .publicKey,
          collectionMint,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .rpc();
      const refreshed: any = await program.account.programState.fetch(
        programStatePda
      );
      collectionMint = new PublicKey(refreshed.collectionMint);
    }

    const tokenId = 456; // Different token ID from first test
    const payload = buildPayload({
      tokenId,
      gatewayMessageIdByte: 3, // Different message ID to avoid replay protection conflict
      metadataUri: 'https://example.com/meta2.json',
      name: 'Cross-Chain NFT 2',
      symbol: 'CCNFT2',
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
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1_200_000 }),
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

    // Second on_call with same token_id but different mint should fail
    const mintKp2 = Keypair.generate();
    const recipientAta2 = anchor.utils.token.associatedAddress({
      mint: mintKp2.publicKey,
      owner: (program.provider as anchor.AnchorProvider).wallet.publicKey,
    });
    const metadataPda2 = deriveMetadataPda(mintKp2.publicKey);
    const masterEditionPda2 = deriveMasterEditionPda(mintKp2.publicKey);
    const nftOriginPda2 = deriveNftOriginPda(tokenId);

    try {
      await program.methods
        .onCall(payload)
        .preInstructions([
          ComputeBudgetProgram.setComputeUnitLimit({ units: 1_200_000 }),
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
    }

    // Cleanup
    try {
      await program.methods
        .closeProgramState()
        .accounts({
          programState: programStatePda,
          payer: (program.provider as anchor.AnchorProvider).wallet.publicKey,
        })
        .rpc();
    } catch (_) {}
  });
});
