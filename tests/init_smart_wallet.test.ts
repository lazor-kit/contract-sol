import * as anchor from "@coral-xyz/anchor";
import { Lazorkit } from "../target/types/lazorkit";
import LazorIdl from "../target/idl/lazorkit.json";
import DefaultRuleIdl from "../target/idl/default_rule.json";
import ECDSA from "ecdsa-secp256r1";
import {
  SMART_WALLET_SEQ_SEED,
  SMART_WALLET_SEED,
  SMART_WALLET_DATA_SEED,
} from "./constants";
import { expect } from "chai";
import {
  Keypair,
  LAMPORTS_PER_SOL,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import { createSecp256r1Instruction, hashSeeds } from "./utils";
import * as dotenv from "dotenv";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { DefaultRule } from "../target/types/default_rule";

dotenv.config();

describe("init_smart_wallet", () => {
  const connection = new anchor.web3.Connection(
    process.env.RPC_URL || "http://localhost:8899",
    "confirmed"
  );

  const lazorProgram = new anchor.Program<Lazorkit>(LazorIdl as Lazorkit, {
    connection,
  });

  const defaultRuleProgram = new anchor.Program<DefaultRule>(
    DefaultRuleIdl as DefaultRule,
    {
      connection,
    }
  );

  const payer = anchor.web3.Keypair.fromSecretKey(
    bs58.decode(process.env.PRIVATE_KEY!)
  );

  const [smartWalletSeq] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from(SMART_WALLET_SEQ_SEED)],
    lazorProgram.programId
  );

  let [authority] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("authority")],
    lazorProgram.programId
  );

  let [defaultRuleConfig] = anchor.web3.PublicKey.findProgramAddressSync(
    [Buffer.from("config")],
    defaultRuleProgram.programId
  );

  before(async () => {
    // airdrop some SOL to the payer

    const smartWalletSeqAccountInfo = await connection.getAccountInfo(
      smartWalletSeq
    );

    if (smartWalletSeqAccountInfo === null) {
      // create the lazor program
      const txn = new anchor.web3.Transaction().add(
        await lazorProgram.methods
          .initialize()
          .accounts({
            signer: payer.publicKey,
            defaultRuleProgram: defaultRuleProgram.programId,
          })
          .instruction()
      );

      await sendAndConfirmTransaction(connection, txn, [payer], {
        commitment: "confirmed",
      });
    }

    const defaultRuleConfigAccountInfo = await connection.getAccountInfo(
      defaultRuleConfig
    );
    if (defaultRuleConfigAccountInfo === null) {
      // create the default rule program
      const txn = new anchor.web3.Transaction().add(
        await defaultRuleProgram.methods
          .initialize(authority)
          .accounts({
            signer: payer.publicKey,
          })
          .instruction()
      );

      await sendAndConfirmTransaction(connection, txn, [payer], {
        commitment: "confirmed",
      });
    }
  });

  it("Initialize successfully", async () => {
    const privateKey = ECDSA.generateKey();

    const publicKeyBase64 = privateKey.toCompressedPublicKey();

    const pubkey = Array.from(Buffer.from(publicKeyBase64, "base64"));

    const SeqBefore = await lazorProgram.account.smartWalletSeq.fetch(
      smartWalletSeq
    );

    const [smartWallet] = anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from(SMART_WALLET_SEED),
        SeqBefore.seq.toArrayLike(Buffer, "le", 8),
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

    // the user has deposit 0.01 SOL to the smart-wallet
    const transferSolIns = anchor.web3.SystemProgram.transfer({
      fromPubkey: payer.publicKey,
      toPubkey: smartWallet,
      lamports: LAMPORTS_PER_SOL / 100,
    });

    await sendAndConfirmTransaction(
      connection,
      new anchor.web3.Transaction().add(transferSolIns),
      [payer],
      {
        commitment: "confirmed",
      }
    );

    const [rule] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("rule"), smartWallet.toBuffer()],
      defaultRuleProgram.programId
    );

    const initRuleIns = await defaultRuleProgram.methods
      .initRule()
      .accountsPartial({
        payer: payer.publicKey,
        lazorkitAuthority: authority,
        smartWallet,
        smartWalletAuthenticator,
        rule,
        config: defaultRuleConfig,
        lazorkit: lazorProgram.programId,
      })
      .instruction();

    // log balance of the payer
    const balance = await connection.getBalance(payer.publicKey);
    console.log("Payer balance:", balance);

    let remainingAccounts: anchor.web3.AccountMeta[] = initRuleIns.keys.map(
      (key) => {
        return {
          pubkey: key.pubkey,
          isWritable: key.isWritable,
          isSigner: key.pubkey === authority ? false : key.isSigner,
        };
      }
    );

    const txn = new anchor.web3.Transaction().add(
      await lazorProgram.methods
        .createSmartWallet(pubkey, initRuleIns.data)
        .accountsPartial({
          signer: payer.publicKey,
          smartWallet,
          smartWalletData,
          smartWalletAuthenticator,
          defaultRuleProgram: defaultRuleProgram.programId,
        })
        .remainingAccounts(remainingAccounts)
        .instruction()
    );

    const sig = await sendAndConfirmTransaction(connection, txn, [payer], {
      commitment: "confirmed",
      skipPreflight: true,
    });

    console.log("Create smart-wallet: ", sig);

    // log balance of the payer
    const balanceAfter = await connection.getBalance(payer.publicKey);
    console.log("Payer balance after:", balanceAfter);

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

  it("Spend SOL successfully with none", async () => {
    const privateKey = ECDSA.generateKey();

    const publicKeyBase64 = privateKey.toCompressedPublicKey();

    const passkeyPubkey = Array.from(Buffer.from(publicKeyBase64, "base64"));

    const SeqBefore = await lazorProgram.account.smartWalletSeq.fetch(
      smartWalletSeq
    );

    const smartWalletSeeds = Buffer.concat([
      Buffer.from(SMART_WALLET_SEED),
      SeqBefore.seq.toArrayLike(Buffer, "le", 8),
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

    // the user has deposit 0.1 SOL to the smart-wallet
    const depositSolIns = anchor.web3.SystemProgram.transfer({
      fromPubkey: payer.publicKey,
      toPubkey: smartWallet,
      lamports: LAMPORTS_PER_SOL / 10,
    });

    await sendAndConfirmTransaction(
      connection,
      new anchor.web3.Transaction().add(depositSolIns),
      [payer],
      {
        commitment: "confirmed",
      }
    );

    const [rule] = anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from("rule"), smartWallet.toBuffer()],
      defaultRuleProgram.programId
    );

    const initRuleIns = await defaultRuleProgram.methods
      .initRule()
      .accountsPartial({
        payer: payer.publicKey,
        lazorkitAuthority: authority,
        smartWallet,
        smartWalletAuthenticator,
        rule,
        config: defaultRuleConfig,
        lazorkit: lazorProgram.programId,
      })
      .instruction();

    let remainingAccounts: anchor.web3.AccountMeta[] = initRuleIns.keys.map(
      (key) => {
        return {
          pubkey: key.pubkey,
          isWritable: key.isWritable,
          isSigner: key.pubkey === authority ? false : key.isSigner,
        };
      }
    );

    const txn = new anchor.web3.Transaction().add(
      await lazorProgram.methods
        .createSmartWallet(passkeyPubkey, initRuleIns.data)
        .accountsPartial({
          signer: payer.publicKey,
          smartWallet,
          smartWalletData,
          smartWalletAuthenticator,
          defaultRuleProgram: defaultRuleProgram.programId,
        })
        .remainingAccounts(remainingAccounts)
        .instruction()
    );

    const createSmartWalletSig = await sendAndConfirmTransaction(
      connection,
      txn,
      [payer],
      {
        commitment: "confirmed",
        skipPreflight: true,
      }
    );

    console.log("Create smart-wallet: ", createSmartWalletSig);

    const message = Buffer.from("hello");
    const signatureBytes = Buffer.from(privateKey.sign(message), "base64");

    const transferSolIns = anchor.web3.SystemProgram.transfer({
      fromPubkey: smartWallet,
      toPubkey: Keypair.generate().publicKey,
      lamports: 5000000,
    });

    remainingAccounts = [];

    let cpiData = {
      data: transferSolIns.data,
      startIndex: 0,
      length: transferSolIns.keys.length,
    };

    remainingAccounts.push(
      ...transferSolIns.keys.map((key) => {
        return {
          pubkey: key.pubkey,
          isWritable: key.isWritable,
          isSigner: key.pubkey === smartWallet ? false : key.isSigner,
        };
      })
    );

    const verifySignatureIns = createSecp256r1Instruction(
      message,
      Buffer.from(passkeyPubkey),
      signatureBytes
    );

    const checkRule = await defaultRuleProgram.methods
      .checkRule()
      .accountsPartial({
        smartWalletAuthenticator,
        rule,
      })
      .instruction();

    let ruleData = {
      data: checkRule.data,
      startIndex: transferSolIns.keys.length,
      length: checkRule.keys.length,
    };

    remainingAccounts.push(
      ...checkRule.keys.map((key) => {
        return {
          pubkey: key.pubkey,
          isWritable: key.isWritable,
          isSigner:
            key.pubkey === smartWalletAuthenticator ? false : key.isSigner,
        };
      })
    );

    const executeTxn = new anchor.web3.Transaction()
      .add(verifySignatureIns)
      .add(
        await lazorProgram.methods
          .executeInstruction({
            passkeyPubkey: passkeyPubkey,
            signature: signatureBytes,
            message,
            verifyInstructionIndex: 0,
            cpiData,
            ruleData,
          })
          .accountsPartial({
            payer: payer.publicKey,
            smartWallet,
            smartWalletData,
            smartWalletAuthenticator,
            ruleProgram: defaultRuleProgram.programId,
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
        commitment: "confirmed",
      }
    );

    console.log("Execute txn: ", sig);
  });
});
