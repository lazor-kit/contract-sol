import * as anchor from "@coral-xyz/anchor";
import { DefaultRule } from "../target/types/default_rule";
import * as types from "./types";
import * as constants from "./constants";

export class DefaultRuleProgram {
  private connection: anchor.web3.Connection;
  private Idl: anchor.Idl = require("../target/idl/default_rule.json");

  constructor(connection: anchor.web3.Connection) {
    this.connection = connection;
  }

  get program(): anchor.Program<DefaultRule> {
    return new anchor.Program(this.Idl, {
      connection: this.connection,
    });
  }

  get programId(): anchor.web3.PublicKey {
    return this.program.programId;
  }

  rule(smartWallet: anchor.web3.PublicKey): anchor.web3.PublicKey {
    return anchor.web3.PublicKey.findProgramAddressSync(
      [constants.RULE_SEED, smartWallet.toBuffer()],
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
    authority: anchor.web3.PublicKey
  ) {
    return new anchor.web3.Transaction().add(
      await this.program.methods
        .initialize(authority)
        .accounts({
          signer: payer,
        })
        .instruction()
    );
  }

  async initRuleIns(
    payer: anchor.web3.PublicKey,
    smartWallet: anchor.web3.PublicKey,
    smartWalletAuthenticator: anchor.web3.PublicKey
  ) {
    const configData = await this.program.account.config.fetch(this.config);
    return await this.program.methods
      .initRule()
      .accountsPartial({
        payer,
        smartWallet,
        smartWalletAuthenticator,
        rule: this.rule(smartWallet),
        systemProgram: anchor.web3.SystemProgram.programId,
      })
      .instruction();
  }

  async checkRuleIns(
    smartWallet: anchor.web3.PublicKey,
    smartWalletAuthenticator: anchor.web3.PublicKey
  ) {
    return await this.program.methods
      .checkRule()
      .accountsPartial({
        rule: this.rule(smartWallet),
        smartWalletAuthenticator,
      })
      .instruction();
  }

  async destroyIns(
    payer: anchor.web3.PublicKey,
    smartWallet: anchor.web3.PublicKey,
    smartWalletAuthenticator: anchor.web3.PublicKey
  ) {
    return await this.program.methods
      .destroy()
      .accountsPartial({
        rule: this.rule(smartWallet),
        smartWalletAuthenticator,
        smartWallet,
      })
      .instruction();
  }
}
