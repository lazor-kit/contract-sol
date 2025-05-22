// import * as anchor from '@coral-xyz/anchor';
// import { Lazorkit } from '../target/types/lazorkit';
// import { TransferLimit } from '../target/types/transfer_limit';
// import LazorIdl from '../target/idl/lazorkit.json';
// import TransferLimitIdl from '../target/idl/transfer_limit.json';
// import ECDSA from 'ecdsa-secp256r1';
// import {
//   SMART_WALLET_SEQ_SEED,
//   SMART_WALLET_SEED,
//   SMART_WALLET_CONFIG_SEED,
//   WHITELIST_RULE_PROGRAMS_SEED,
//   RULE_DATA_SEED,
//   MEMBER_SEED,
// } from './constants';
// import { expect } from 'chai';
// import { LAMPORTS_PER_SOL, sendAndConfirmTransaction } from '@solana/web3.js';
// import { createSecp256r1Instruction, fundAccountSOL, hashSeeds } from './utils';
// import * as dotenv from 'dotenv';
// import { bs58 } from '@coral-xyz/anchor/dist/cjs/utils/bytes';

// dotenv.config();

// describe.skip('add_member', () => {
//   const connection = new anchor.web3.Connection(
//     process.env.RPC_URL || 'http://localhost:8899',
//     'confirmed'
//   );

//   const lazorProgram = new anchor.Program<Lazorkit>(LazorIdl as Lazorkit, {
//     connection,
//   });

//   const transferLimitProgram = new anchor.Program<TransferLimit>(
//     TransferLimitIdl as TransferLimit,
//     {
//       connection,
//     }
//   );

//   const payer = anchor.web3.Keypair.fromSecretKey(
//     bs58.decode(process.env.PRIVATE_KEY!)
//   );

//   const [smartWalletSeq] = anchor.web3.PublicKey.findProgramAddressSync(
//     [Buffer.from(SMART_WALLET_SEQ_SEED)],
//     lazorProgram.programId
//   );
//   let smartWallet: anchor.web3.PublicKey;
//   let smartWalletConfig: anchor.web3.PublicKey;
//   let smartWalletAuthenticator: anchor.web3.PublicKey;
//   let passkeyKeypair: ECDSA.Key;
//   let passkeyPubkey: number[];
//   let adminMember: anchor.web3.PublicKey;

//   before(async () => {
//     const [whitelistRulePrograms] =
//       anchor.web3.PublicKey.findProgramAddressSync(
//         [Buffer.from(WHITELIST_RULE_PROGRAMS_SEED)],
//         lazorProgram.programId
//       );

//     const data = await connection.getAccountInfo(whitelistRulePrograms);

//     if (data) {
//       const whitelistRuleProgramsData =
//         await lazorProgram.account.whitelistRulePrograms.fetch(
//           whitelistRulePrograms
//         );

//       // check if the whitelist rule programs is empty
//       if (
//         !whitelistRuleProgramsData.list.includes(transferLimitProgram.programId)
//       ) {
//         const txn = new anchor.web3.Transaction().add(
//           await lazorProgram.methods
//             .upsertWhitelistRulePrograms(transferLimitProgram.programId)
//             .accountsPartial({
//               signer: payer.publicKey,
//               whitelistRulePrograms,
//             })
//             .instruction()
//         );

//         await sendAndConfirmTransaction(connection, txn, [payer], {
//           commitment: 'confirmed',
//         });
//       }
//     } else {
//       // create the lazor program
//       const txn = new anchor.web3.Transaction().add(
//         await lazorProgram.methods
//           .initialize()
//           .accounts({
//             signer: payer.publicKey,
//           })
//           .instruction()
//       );

//       await sendAndConfirmTransaction(connection, txn, [payer], {
//         commitment: 'confirmed',
//       });

//       const upsertWhitelistRuleProgramsTxn = new anchor.web3.Transaction().add(
//         await lazorProgram.methods
//           .upsertWhitelistRulePrograms(transferLimitProgram.programId)
//           .accountsPartial({
//             signer: payer.publicKey,
//             whitelistRulePrograms,
//           })
//           .instruction()
//       );
//       await sendAndConfirmTransaction(
//         connection,
//         upsertWhitelistRuleProgramsTxn,
//         [payer],
//         {
//           commitment: 'confirmed',
//         }
//       );
//     }

//     passkeyKeypair = ECDSA.generateKey();

//     passkeyPubkey = Array.from(
//       Buffer.from(passkeyKeypair.toCompressedPublicKey(), 'base64')
//     );

//     let currentSmartWalletSeq = await lazorProgram.account.smartWalletSeq.fetch(
//       smartWalletSeq
//     );

//     smartWallet = anchor.web3.PublicKey.findProgramAddressSync(
//       [
//         Buffer.from(SMART_WALLET_SEED),
//         currentSmartWalletSeq.seq.toArrayLike(Buffer, 'le', 8),
//       ],
//       lazorProgram.programId
//     )[0];

//     smartWalletConfig = anchor.web3.PublicKey.findProgramAddressSync(
//       [Buffer.from(SMART_WALLET_CONFIG_SEED), smartWallet.toBuffer()],
//       lazorProgram.programId
//     )[0];

//     smartWalletAuthenticator = anchor.web3.PublicKey.findProgramAddressSync(
//       [hashSeeds(passkeyPubkey, smartWallet)],
//       lazorProgram.programId
//     )[0];

//     // the user has deposit 0.01 SOL to the smart-wallet
//     const transferSolIns = anchor.web3.SystemProgram.transfer({
//       fromPubkey: payer.publicKey,
//       toPubkey: smartWallet,
//       lamports: LAMPORTS_PER_SOL / 100,
//     });

//     await sendAndConfirmTransaction(
//       connection,
//       new anchor.web3.Transaction().add(transferSolIns),
//       [payer],
//       {
//         commitment: 'confirmed',
//       }
//     );

//     const txn = new anchor.web3.Transaction().add(
//       await lazorProgram.methods
//         .createSmartWallet(passkeyPubkey)
//         .accountsPartial({
//           signer: payer.publicKey,
//           smartWallet,
//           smartWalletConfig,
//           smartWalletAuthenticator,
//         })
//         .instruction()
//     );

//     await sendAndConfirmTransaction(connection, txn, [payer], {
//       commitment: 'confirmed',
//       skipPreflight: true,
//     });

//     adminMember = anchor.web3.PublicKey.findProgramAddressSync(
//       [
//         Buffer.from(MEMBER_SEED),
//         smartWallet.toBuffer(),
//         smartWalletAuthenticator.toBuffer(),
//       ],
//       transferLimitProgram.programId
//     )[0];

//     const [ruleData] = anchor.web3.PublicKey.findProgramAddressSync(
//       [
//         Buffer.from(RULE_DATA_SEED),
//         smartWallet.toBuffer(),
//         anchor.web3.PublicKey.default.toBuffer(),
//       ],
//       transferLimitProgram.programId
//     );

//     const initTransferLimitIns = await transferLimitProgram.methods
//       .initRule({
//         passkeyPubkey: passkeyPubkey,
//         token: null,
//         limitAmount: new anchor.BN(LAMPORTS_PER_SOL),
//         limitPeriod: new anchor.BN(0),
//       })
//       .accountsPartial({
//         payer: payer.publicKey,
//         smartWallet,
//         smartWalletConfig,
//         smartWalletAuthenticator,
//         member: adminMember,
//         ruleData,
//       })
//       .instruction();

//     const message = Buffer.from('Hello');
//     const signatureBytes = Buffer.from(passkeyKeypair.sign(message), 'base64');

//     const verifySignatureIns = createSecp256r1Instruction(
//       message,
//       Buffer.from(passkeyPubkey),
//       signatureBytes
//     );

//     const executeTxn = new anchor.web3.Transaction()
//       .add(verifySignatureIns)
//       .add(
//         await lazorProgram.methods
//           .executeInstruction({
//             passkeyPubkey: passkeyPubkey,
//             cpiData: initTransferLimitIns.data,
//             signature: signatureBytes,
//             message,
//             verifyInstructionIndex: 0,
//             cpiSigner: null,
//             callRule: true,
//           })
//           .accountsPartial({
//             payer: payer.publicKey,
//             smartWallet,
//             smartWalletConfig,
//             smartWalletAuthenticator,
//             cpiProgram: transferLimitProgram.programId,
//           })
//           .remainingAccounts(initTransferLimitIns.keys)
//           .instruction()
//       );

//     await sendAndConfirmTransaction(connection, executeTxn, [payer], {
//       commitment: 'confirmed',
//     });
//   });

//   it('Add member', async () => {
//     const newPasskeyKeypair = ECDSA.generateKey();
//     const newPasskeyPubkey = Array.from(
//       Buffer.from(newPasskeyKeypair.toCompressedPublicKey(), 'base64')
//     );

//     const [newSmartWalletAuthenticator, newSmartWalletAuthenticatorBump] =
//       anchor.web3.PublicKey.findProgramAddressSync(
//         [hashSeeds(newPasskeyPubkey, smartWallet)],
//         lazorProgram.programId
//       );

//     const [member] = anchor.web3.PublicKey.findProgramAddressSync(
//       [
//         Buffer.from(MEMBER_SEED),
//         smartWallet.toBuffer(),
//         newSmartWalletAuthenticator.toBuffer(),
//       ],
//       transferLimitProgram.programId
//     );

//     const addMemberIns = await transferLimitProgram.methods
//       .addMember(newPasskeyPubkey)
//       .accountsPartial({
//         payer: payer.publicKey,
//         smartWalletAuthenticator,
//         newSmartWalletAuthenticator,
//         admin: adminMember,
//         member,
//       })
//       .instruction();

//     const message = Buffer.from('Hello');
//     const signatureBytes = Buffer.from(passkeyKeypair.sign(message), 'base64');

//     const verifySignatureIns = createSecp256r1Instruction(
//       message,
//       Buffer.from(passkeyPubkey),
//       signatureBytes
//     );

//     let remainingAccounts: anchor.web3.AccountMeta[] = addMemberIns.keys.map(
//       (key) => {
//         return {
//           pubkey: key.pubkey,
//           isWritable: key.isWritable,
//           isSigner:
//             key.pubkey === newSmartWalletAuthenticator ? false : key.isSigner,
//         };
//       }
//     );

//     const executeIns = await lazorProgram.methods
//       .executeInstruction({
//         passkeyPubkey: passkeyPubkey,
//         cpiData: addMemberIns.data,
//         signature: signatureBytes,
//         message,
//         verifyInstructionIndex: 0,
//         cpiSigner: {
//           seeds: Buffer.from(hashSeeds(newPasskeyPubkey, smartWallet)),
//           bump: newSmartWalletAuthenticatorBump,
//         },
//         callRule: true,
//       })
//       .accountsPartial({
//         payer: payer.publicKey,
//         smartWallet,
//         smartWalletConfig,
//         smartWalletAuthenticator,
//         cpiProgram: transferLimitProgram.programId,
//       })
//       .remainingAccounts(remainingAccounts)
//       .instruction();

//     const txn = new anchor.web3.Transaction()
//       .add(verifySignatureIns)
//       .add(executeIns);

//     const sig = await sendAndConfirmTransaction(connection, txn, [payer], {
//       commitment: 'confirmed',
//       skipPreflight: true,
//     });

//     console.log(sig);
//   });
// });
