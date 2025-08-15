import * as anchor from '@coral-xyz/anchor';
import { Program } from '@coral-xyz/anchor';
import { SystemProgram, PublicKey, clusterApiUrl } from '@solana/web3.js';
import { UniversalNft } from '../target/types/universal_nft';

describe('initialize_program_state on devnet', () => {
  const connection = new anchor.web3.Connection(
    process.env.ANCHOR_PROVIDER_URL || clusterApiUrl('devnet'),
    'confirmed'
  );

  const wallet = anchor.Wallet.local();
  const provider = new anchor.AnchorProvider(connection, wallet, {});
  anchor.setProvider(provider);

  const program = anchor.workspace.universalNft as Program<UniversalNft>;

  it('initializes the program state PDA', async () => {
    const [programStatePda] = PublicKey.findProgramAddressSync(
      [Buffer.from('program-state')],
      program.programId
    );

    const sig = await program.methods
      .initializeProgramState()
      .accounts({
        payer: wallet.publicKey,
      })
      .rpc();

    console.log('Program state initialized:', programStatePda.toBase58());
    console.log('Signature:', sig);
  });
});
