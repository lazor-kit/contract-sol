import * as anchor from "@coral-xyz/anchor";
import IDL from "../target/idl/lazorkit.json";
import { Lazorkit } from "../target/types/lazorkit";
import * as constants from "./constants";
import { createSecp256r1Instruction, hashSeeds } from "./utils";
import * as types from "./types";

export class LazorKitProgram {
  readonly connection: anchor.web3.Connection;
  readonly Idl: anchor.Idl = IDL as Lazorkit;

  constructor(connection: anchor.web3.Connection) {
    this.connection = connection;
  }

  get program(): anchor.Program<Lazorkit> {
    return new anchor.Program(this.Idl, { connection: this.connection });
  }

  get programId(): anchor.web3.PublicKey {
    return this.program.programId;
  }

  get smartWalletSeq(): anchor.web3.PublicKey {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [constants.SMART_WALLET_SEQ_SEED],
      this.programId
    )[0];
  }

  get smartWalletSeqData(): Promise<types.SmartWalletSeq> {
    return this.program.account.smartWalletSeq.fetch(this.smartWalletSeq);
  }

  get authority(): anchor.web3.PublicKey {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [constants.AUTHORITY_SEED],
      this.programId
    )[0];
  }

  async getLastestSmartWallet(): Promise<anchor.web3.PublicKey> {
    const seqData = await this.program.account.smartWalletSeq.fetch(
      this.smartWalletSeq
    );
    return anchor.web3.PublicKey.findProgramAddressSync(
      [constants.SMART_WALLET_SEED, seqData.seq.toArrayLike(Buffer, "le", 8)],
      this.programId
    )[0];
  }

  async getSmartWalletConfigData(
    smartWallet: anchor.web3.PublicKey
  ): Promise<types.SmartWalletConfig> {
    return this.program.account.smartWalletConfig.fetch(
      this.smartWalletConfig(smartWallet)
    );
  }

  smartWalletAuthenticator(
    passkey: number[],
    smartWallet: anchor.web3.PublicKey
  ): anchor.web3.PublicKey {
    const hash = hashSeeds(passkey, smartWallet);
    return anchor.web3.PublicKey.findProgramAddressSync(
      [hash],
      this.programId
    )[0];
  }

  async getSmartWalletAuthenticatorData(
    smartWalletAuthenticator: anchor.web3.PublicKey
  ): Promise<types.SmartWalletAuthenticator> {
    return this.program.account.smartWalletAuthenticator.fetch(
      smartWalletAuthenticator
    );
  }

  smartWalletConfig(smartWallet: anchor.web3.PublicKey): anchor.web3.PublicKey {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [constants.SMART_WALLET_CONFIG_SEED, smartWallet.toBuffer()],
      this.programId
    )[0];
  }

  get whitelistRulePrograms(): anchor.web3.PublicKey {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [constants.WHITELIST_RULE_PROGRAMS_SEED],
      this.programId
    )[0];
  }

  get config(): anchor.web3.PublicKey {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [constants.CONFIG_SEED],
      this.programId
    )[0];
  }

  async initializeTxn(
    payer: anchor.web3.PublicKey,
    defaultRuleProgram: anchor.web3.PublicKey
  ): Promise<anchor.web3.Transaction> {
    const ix = await this.program.methods
      .initialize()
      .accountsPartial({
        signer: payer,
        config: this.config,
        whitelistRulePrograms: this.whitelistRulePrograms,
        smartWalletSeq: this.smartWalletSeq,
        authority: this.authority,
        defaultRuleProgram,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .instruction();
    return new anchor.web3.Transaction().add(ix);
  }

  async upsertWhitelistRuleProgramsTxn(
    payer: anchor.web3.PublicKey,
    ruleProgram: anchor.web3.PublicKey
  ): Promise<anchor.web3.Transaction> {
    const ix = await this.program.methods
      .upsertWhitelistRulePrograms(ruleProgram)
      .accountsPartial({
        signer: payer,
        whitelistRulePrograms: this.whitelistRulePrograms,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .instruction();
    return new anchor.web3.Transaction().add(ix);
  }

  async createSmartWalletTxn(
    passkeyPubkey: number[],
    ruleIns: anchor.web3.TransactionInstruction,
    payer: anchor.web3.PublicKey
  ): Promise<anchor.web3.Transaction> {
    const configData = await this.program.account.config.fetch(this.config);
    const smartWallet = await this.getLastestSmartWallet();
    const smartWalletAuthenticator = this.smartWalletAuthenticator(
      passkeyPubkey,
      smartWallet
    );

    const remainingAccounts = ruleIns.keys.map((account) => ({
      pubkey: account.pubkey,
      isSigner: account.pubkey.equals(this.authority)
        ? false
        : account.isSigner,
      isWritable: account.isWritable,
    }));

    const createSmartWalletIx = await this.program.methods
      .createSmartWallet(passkeyPubkey, ruleIns.data)
      .accountsPartial({
        signer: payer,
        smartWalletSeq: this.smartWalletSeq,
        whitelistRulePrograms: this.whitelistRulePrograms,
        smartWallet,
        smartWalletConfig: this.smartWalletConfig(smartWallet),
        smartWalletAuthenticator,
        config: this.config,
        defaultRuleProgram: configData.defaultRuleProgram,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .remainingAccounts(remainingAccounts)
      .instruction();

    const tx = new anchor.web3.Transaction().add(createSmartWalletIx);
    tx.feePayer = payer;
    tx.recentBlockhash = (await this.connection.getLatestBlockhash()).blockhash;
    return tx;
  }

  async executeInstructionTxn(
    passkeyPubkey: number[],
    message: Buffer,
    signature: Buffer,
    authenticationIns: anchor.web3.TransactionInstruction,
    cpiIns: anchor.web3.TransactionInstruction,
    payer: anchor.web3.PublicKey,
    smartWallet: anchor.web3.PublicKey,
    executeAction: anchor.IdlTypes<Lazorkit>["action"] = types.ExecuteAction
      .ExecuteCpi,
    verifyInstructionIndex: number = 0
  ): Promise<anchor.web3.Transaction> {
    const smartWalletAuthenticator = this.smartWalletAuthenticator(
      passkeyPubkey,
      smartWallet
    );

    const authenticationData: types.CpiData = {
      data: authenticationIns.data,
      startIndex: 0,
      length: authenticationIns.keys.length,
    };

    const cpiData: types.CpiData = {
      data: cpiIns.data,
      startIndex: authenticationIns.keys.length,
      length: cpiIns.keys.length,
    };

    const remainingAccounts: anchor.web3.AccountMeta[] = [
      ...authenticationIns.keys.map((key) => ({
        pubkey: key.pubkey,
        isWritable: key.isWritable,
        isSigner: key.pubkey.equals(smartWalletAuthenticator)
          ? false
          : key.isSigner,
      })),
      ...cpiIns.keys.map((key) => ({
        pubkey: key.pubkey,
        isWritable: key.isWritable,
        isSigner: key.pubkey === payer,
      })),
    ];

    const verifySignatureIx = createSecp256r1Instruction(
      message,
      Buffer.from(passkeyPubkey),
      signature
    );

    const executeInstructionIx = await this.program.methods
      .executeInstruction({
        passkeyPubkey,
        signature,
        message,
        verifyInstructionIndex,
        ruleData: authenticationData,
        cpiData,
        action: executeAction,
      })
      .accountsPartial({
        payer,
        smartWallet,
        smartWalletConfig: this.smartWalletConfig(smartWallet),
        smartWalletAuthenticator,
        whitelistRulePrograms: this.whitelistRulePrograms,
        cpiProgram: cpiIns.programId,
        authenticatorProgram: authenticationIns.programId,
        ixSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .remainingAccounts(remainingAccounts)
      .instruction();

    return new anchor.web3.Transaction()
      .add(verifySignatureIx)
      .add(executeInstructionIx);
  }
}
