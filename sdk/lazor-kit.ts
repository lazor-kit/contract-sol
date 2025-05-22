import * as anchor from "@coral-xyz/anchor";
import IDL from "../target/idl/lazorkit.json";
import { Lazorkit } from "../target/types/lazorkit";
import * as constants from "./constants";
import { createSecp256r1Instruction, hashSeeds } from "./utils";
import * as types from "./types";

export class LazorKitProgram {
  private connection: anchor.web3.Connection;
  private Idl: anchor.Idl = IDL as Lazorkit;

  constructor(connection: anchor.web3.Connection) {
    this.connection = connection;
  }

  get program(): anchor.Program<Lazorkit> {
    return new anchor.Program(this.Idl, {
      connection: this.connection,
    });
  }

  get programId(): anchor.web3.PublicKey {
    return this.program.programId;
  }

  get smartWalletSeq(): anchor.web3.PublicKey {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [Buffer.from(constants.SMART_WALLET_SEQ_SEED)],
      this.programId
    )[0];
  }

  get smartWalletSeqData(): Promise<types.SmartWalletSeq> {
    return Promise.resolve(
      this.program.account.smartWalletSeq.fetch(this.smartWalletSeq)
    );
  }

  get authority(): anchor.web3.PublicKey {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [constants.AUTHORITY_SEED],
      this.programId
    )[0];
  }

  async getLastestSmartWallet(): Promise<anchor.web3.PublicKey> {
    const smartWalletSeqData = await this.program.account.smartWalletSeq.fetch(
      this.smartWalletSeq
    );

    return anchor.web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from(constants.SMART_WALLET_SEED),
        smartWalletSeqData.seq.toArrayLike(Buffer, "le", 8),
      ],
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
    return new anchor.web3.Transaction().add(
      await this.program.methods
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
        .instruction()
    );
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

    const remainingAccounts = ruleIns.keys.map((account) => {
      return {
        pubkey: account.pubkey,
        isSigner:
          account.pubkey.toString() === this.authority.toString()
            ? false
            : account.isSigner,
        isWritable: account.isWritable,
      };
    });

    const createSmartWalletIns = await this.program.methods
      .createSmartWallet(passkeyPubkey, ruleIns.data)
      .accountsPartial({
        signer: payer,
        smartWalletSeq: this.smartWalletSeq,
        whitelistRulePrograms: this.whitelistRulePrograms,
        smartWallet: await this.getLastestSmartWallet(),
        smartWalletConfig: this.smartWalletConfig(smartWallet),
        smartWalletAuthenticator,
        config: this.config,
        defaultRuleProgram: configData.defaultRuleProgram,
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .remainingAccounts(remainingAccounts)
      .instruction();

    const transaction = new anchor.web3.Transaction().add(createSmartWalletIns);
    transaction.feePayer = payer;
    transaction.recentBlockhash = (
      await this.connection.getLatestBlockhash()
    ).blockhash;

    return transaction;
  }

  async executeInstructionTxn(
    passkeyPubkey: number[],
    message: Buffer,
    signature: Buffer,
    checkRuleIns: anchor.web3.TransactionInstruction,
    cpiIns: anchor.web3.TransactionInstruction,
    payer: anchor.web3.PublicKey,
    smartWallet: anchor.web3.PublicKey,
    verifyInstructionIndex?: number
  ) {
    let remainingAccounts: anchor.web3.AccountMeta[] = [];
    const smartWalletAuthenticator = this.smartWalletAuthenticator(
      passkeyPubkey,
      smartWallet
    );

    const checkRuleData: types.CpiData = {
      data: checkRuleIns.data,
      startIndex: 0,
      length: checkRuleIns.keys.length,
    };

    remainingAccounts.push(
      ...cpiIns.keys.map((key) => {
        return {
          pubkey: key.pubkey,
          isWritable: key.isWritable,
          isSigner:
            key.pubkey === smartWalletAuthenticator ? false : key.isSigner,
        };
      })
    );

    const cpiData: types.CpiData = {
      data: cpiIns.data,
      startIndex: checkRuleIns.keys.length,
      length: cpiIns.keys.length,
    };

    remainingAccounts.push(
      ...cpiIns.keys.map((key) => {
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
      signature
    );

    const executeInstructionIns = await this.program.methods
      .executeInstruction({
        passkeyPubkey,
        signature,
        message,
        verifyInstructionIndex: verifyInstructionIndex
          ? verifyInstructionIndex
          : 0,
        cpiData: cpiData,
        ruleData: checkRuleData,
      })
      .accountsPartial({
        payer,
        smartWallet,
        smartWalletConfig: this.smartWalletConfig(smartWallet),
        smartWalletAuthenticator,
        whitelistRulePrograms: this.whitelistRulePrograms,
        cpiProgram: cpiIns.programId,
        ruleProgram: checkRuleIns.programId,
        ixSysvar: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY,
      })
      .remainingAccounts(remainingAccounts)
      .instruction();

    return new anchor.web3.Transaction().add(verifySignatureIns);
  }
}
