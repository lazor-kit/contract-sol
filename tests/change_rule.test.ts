import * as anchor from "@coral-xyz/anchor";
import ECDSA from "ecdsa-secp256r1";
import { LAMPORTS_PER_SOL, sendAndConfirmTransaction } from "@solana/web3.js";
import * as dotenv from "dotenv";
import { bs58 } from "@coral-xyz/anchor/dist/cjs/utils/bytes";
import { LazorKitProgram } from "../sdk/lazor-kit";
import { DefaultRuleProgram } from "../sdk/default-rule-program";

import { ExecuteAction } from "../sdk/types";
import { TransferLimitProgram } from "../sdk/transfer_limit";
dotenv.config();

describe("Test smart wallet with default rule", () => {
  const connection = new anchor.web3.Connection(
    process.env.RPC_URL || "http://localhost:8899",
    "confirmed"
  );

  const lazorkitProgram = new LazorKitProgram(connection);

  const defaultRuleProgram = new DefaultRuleProgram(connection);

  const transferLimitProgram = new TransferLimitProgram(connection);

  const payer = anchor.web3.Keypair.fromSecretKey(
    bs58.decode(process.env.PRIVATE_KEY!)
  );

  before(async () => {
    const smartWalletSeqAccountInfo = await connection.getAccountInfo(
      lazorkitProgram.smartWalletSeq
    );

    if (smartWalletSeqAccountInfo === null) {
      const txn = await lazorkitProgram.initializeTxn(
        payer.publicKey,
        defaultRuleProgram.programId
      );

      await sendAndConfirmTransaction(connection, txn, [payer], {
        commitment: "confirmed",
      });
    }

    const defaultRuleConfigAccountInfo = await connection.getAccountInfo(
      defaultRuleProgram.config
    );

    if (defaultRuleConfigAccountInfo === null) {
      // create the default rule program
      const txn = await defaultRuleProgram.initializeTxn(
        payer.publicKey,
        lazorkitProgram.authority
      );

      await sendAndConfirmTransaction(connection, txn, [payer], {
        commitment: "confirmed",
      });
    }

    const transferLimitConfigAccountInfo = await connection.getAccountInfo(
      transferLimitProgram.config
    );
    if (transferLimitConfigAccountInfo === null) {
      // create the transfer limit program
      const txn = await transferLimitProgram.initializeTxn(
        payer.publicKey,
        lazorkitProgram.authority
      );

      await sendAndConfirmTransaction(connection, txn, [payer], {
        commitment: "confirmed",
      });
    }

    const whitelistRuleProgramData =
      await lazorkitProgram.program.account.whitelistRulePrograms.fetch(
        lazorkitProgram.whitelistRulePrograms
      );

    // check if already have transfer limit program
    if (
      !whitelistRuleProgramData.list.includes(transferLimitProgram.programId)
    ) {
      const txn = await lazorkitProgram.upsertWhitelistRuleProgramsTxn(
        payer.publicKey,
        transferLimitProgram.programId
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

    const smartWallet = await lazorkitProgram.getLastestSmartWallet();

    const smartWalletConfig = lazorkitProgram.smartWalletConfig(smartWallet);

    const smartWalletAuthenticator = lazorkitProgram.smartWalletAuthenticator(
      pubkey,
      smartWallet
    );

    // the user has deposit 0.01 SOL to the smart-wallet
    const depositSolIns = anchor.web3.SystemProgram.transfer({
      fromPubkey: payer.publicKey,
      toPubkey: smartWallet,
      lamports: LAMPORTS_PER_SOL / 100,
    });

    await sendAndConfirmTransaction(
      connection,
      new anchor.web3.Transaction().add(depositSolIns),
      [payer],
      {
        commitment: "confirmed",
      }
    );

    const initRuleIns = await defaultRuleProgram.initRuleIns(
      payer.publicKey,
      smartWallet,
      smartWalletAuthenticator
    );

    const createSmartWalletTxn = await lazorkitProgram.createSmartWalletTxn(
      pubkey,
      initRuleIns,
      payer.publicKey
    );

    const sig = await sendAndConfirmTransaction(
      connection,
      createSmartWalletTxn,
      [payer],
      {
        commitment: "confirmed",
        skipPreflight: true,
      }
    );

    console.log("Create smart-wallet: ", sig);

    // Change rule
    const destroyRuleDefaultIns = await defaultRuleProgram.destroyIns(
      payer.publicKey,
      smartWallet,
      smartWalletAuthenticator
    );

    const initTransferLimitRule = await transferLimitProgram.initRuleIns(
      payer.publicKey,
      smartWallet,
      smartWalletAuthenticator,
      smartWalletConfig,
      {
        passkeyPubkey: pubkey,
        token: anchor.web3.PublicKey.default,
        limitAmount: new anchor.BN(100),
        limitPeriod: new anchor.BN(1000),
      }
    );

    const message = Buffer.from("hello");
    const signatureBytes = Buffer.from(privateKey.sign(message), "base64");

    console.log(lazorkitProgram.authority.toBase58());

    const executeTxn = await lazorkitProgram.executeInstructionTxn(
      pubkey,
      message,
      signatureBytes,
      destroyRuleDefaultIns,
      initTransferLimitRule,
      payer.publicKey,
      smartWallet,
      ExecuteAction.ChangeRule
    );

    const sig2 = await sendAndConfirmTransaction(
      connection,
      executeTxn,
      [payer],
      {
        commitment: "confirmed",
        skipPreflight: true,
      }
    );
    console.log("Execute instruction: ", sig2);
  });
});
