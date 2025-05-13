import * as anchor from '@coral-xyz/anchor';
import { Lazorkit } from '../target/types/lazorkit';
import LazorIdl from '../target/idl/lazorkit.json';
import ECDSA from 'ecdsa-secp256r1';
import {
  SMART_WALLET_SEQ_SEED,
  SMART_WALLET_SEED,
  SMART_WALLET_DATA_SEED,
} from './constants';
import { expect } from 'chai';
import {
  Keypair,
  LAMPORTS_PER_SOL,
  PublicKey,
  sendAndConfirmTransaction,
  SystemProgram,
} from '@solana/web3.js';
import { createSecp256r1Instruction, fundAccountSOL, hashSeeds } from './utils';

describe('init_smart_wallet', () => {
  const connection = new anchor.web3.Connection(
    'http://localhost:8899',
    'confirmed'
  );

  const lazorProgram = new anchor.Program<Lazorkit>(LazorIdl as Lazorkit, {
    connection,
  });

  const payer = anchor.web3.Keypair.generate();

  const [smartWalletSeq] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(SMART_WALLET_SEQ_SEED)],
    lazorProgram.programId
  );

  before(async () => {
    // airdrop some SOL to the payer
    await fundAccountSOL(connection, payer.publicKey, LAMPORTS_PER_SOL * 10);

    try {
      // create the lazor program
      const txn = new anchor.web3.Transaction().add(
        await lazorProgram.methods
          .initialize()
          .accounts({
            signer: payer.publicKey,
          })
          .instruction()
      );

      await sendAndConfirmTransaction(connection, txn, [payer], {
        commitment: 'confirmed',
      });
    } catch (error) {}
  });

  xit('Initialize successfully', async () => {
    const privateKey = ECDSA.generateKey();

    const publicKeyBase64 = privateKey.toCompressedPublicKey();

    const pubkey = Array.from(Buffer.from(publicKeyBase64, 'base64'));

    const SeqBefore = await lazorProgram.account.smartWalletSeq.fetch(
      smartWalletSeq
    );

    const [smartWallet] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from(SMART_WALLET_SEED),
        SeqBefore.seq.toArrayLike(Buffer, 'le', 8),
      ],
      lazorProgram.programId
    );

    const [smartWalletData] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from(SMART_WALLET_DATA_SEED), smartWallet.toBuffer()],
      lazorProgram.programId
    );

    const [smartWalletAuthenticator] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [hashSeeds(pubkey, smartWallet)],
        lazorProgram.programId
      );

    const txn = new anchor.web3.Transaction().add(
      await lazorProgram.methods
        .createSmartWallet(pubkey)
        .accountsPartial({
          signer: payer.publicKey,
          smartWallet,
          smartWalletData,
          smartWalletAuthenticator,
        })
        .instruction()
    );

    await sendAndConfirmTransaction(connection, txn, [payer], {
      commitment: 'confirmed',
    });

    const SeqAfter = await lazorProgram.account.smartWalletSeq.fetch(
      smartWalletSeq
    );

    expect(SeqAfter.seq.toString()).to.be.equal(
      SeqBefore.seq.add(new anchor.BN(1)).toString()
    );

    const smartWalletDataData =
      await lazorProgram.account.smartWalletData.fetch(smartWalletData);

    expect(smartWalletDataData.id.toString()).to.be.equal(
      SeqBefore.seq.toString()
    );

    const smartWalletAuthenticatorData =
      await lazorProgram.account.smartWalletAuthenticator.fetch(
        smartWalletAuthenticator
      );

    expect(smartWalletAuthenticatorData.passkeyPubkey.toString()).to.be.equal(
      pubkey.toString()
    );
    expect(smartWalletAuthenticatorData.smartWallet.toString()).to.be.equal(
      smartWallet.toString()
    );
  });

  it('Spend SOL successfully', async () => {
    const privateKey = ECDSA.generateKey();

    const publicKeyBase64 = privateKey.toCompressedPublicKey();

    const passkeyPubkey = Array.from(Buffer.from(publicKeyBase64, 'base64'));

    const SeqBefore = await lazorProgram.account.smartWalletSeq.fetch(
      smartWalletSeq
    );

    const smartWalletSeeds = Buffer.concat([
      Buffer.from(SMART_WALLET_SEED),
      SeqBefore.seq.toArrayLike(Buffer, 'le', 8),
    ]);

    const [smartWallet, smartWalletBump] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [smartWalletSeeds],
        lazorProgram.programId
      );

    const [smartWalletData] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from(SMART_WALLET_DATA_SEED), smartWallet.toBuffer()],
      lazorProgram.programId
    );

    const [smartWalletAuthenticator] =
      anchor.web3.PublicKey.findProgramAddressSync(
        [hashSeeds(passkeyPubkey, smartWallet)],
        lazorProgram.programId
      );

    const txn = new anchor.web3.Transaction().add(
      await lazorProgram.methods
        .createSmartWallet(passkeyPubkey)
        .accountsPartial({
          signer: payer.publicKey,
          smartWallet,
          smartWalletData,
          smartWalletAuthenticator,
        })
        .instruction()
    );

    const createSmartWalletSig = await sendAndConfirmTransaction(
      connection,
      txn,
      [payer],
      {
        commitment: 'confirmed',
        skipPreflight: true,
      }
    );

    console.log(createSmartWalletSig);

    // fund the smart wallet
    await fundAccountSOL(connection, smartWallet, LAMPORTS_PER_SOL);

    const message = Buffer.from('hello');
    const signatureBytes = Buffer.from(privateKey.sign(message), 'base64');

    const transferSolIns = anchor.web3.SystemProgram.transfer({
      fromPubkey: smartWallet,
      toPubkey: Keypair.generate().publicKey,
      lamports: LAMPORTS_PER_SOL,
    });

    const remainingAccounts = transferSolIns.keys.map((key) => {
      return {
        pubkey: key.pubkey,
        isWritable: key.isWritable,
        isSigner: key.pubkey === smartWallet ? false : key.isSigner,
      };
    });

    const verifySignatureIns = createSecp256r1Instruction(
      message,
      Buffer.from(passkeyPubkey),
      signatureBytes
    );

    const executeTxn = new anchor.web3.Transaction()
      .add(verifySignatureIns)
      .add(
        await lazorProgram.methods
          .executeInstruction({
            passkeyPubkey: passkeyPubkey,
            cpiData: transferSolIns.data,
            signature: signatureBytes,
            message,
            verifyInstructionIndex: 0,
            cpiSigner: {
              seeds: smartWalletSeeds,
              bump: smartWalletBump,
            },
            callRule: false,
          })
          .accountsPartial({
            payer: payer.publicKey,
            smartWallet,
            smartWalletData,
            smartWalletAuthenticator,
            cpiProgram: anchor.web3.SystemProgram.programId,
          })
          .remainingAccounts(remainingAccounts)
          .instruction()
      );

    const sig = await sendAndConfirmTransaction(
      connection,
      executeTxn,
      [payer],
      {
        commitment: 'confirmed',
      }
    );

    console.log(sig);
  });
});
