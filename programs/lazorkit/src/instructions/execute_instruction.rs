use anchor_lang::{prelude::*, solana_program::sysvar::instructions::load_instruction_at_checked};

use crate::{
    constants::SMART_WALLET_SEED,
    error::LazorKitError,
    state::{SmartWalletAuthenticator, SmartWalletData, WhitelistRulePrograms},
    utils::{execute_cpi_instruction, verify_secp256r1_ix},
    ID,
};
use anchor_lang::solana_program::instruction::Instruction;
use anchor_lang::solana_program::sysvar::instructions::ID as IX_ID;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExecuteInstructionArgs {
    pub cpi_data: Vec<u8>,
    pub signature: Vec<u8>,
    pub message: Vec<u8>,
    pub verify_instruction_index: u8,
}

pub fn execute_instruction(
    ctx: Context<ExecuteInstruction>,
    args: ExecuteInstructionArgs,
) -> Result<()> {
    let wallet_data = &ctx.accounts.smart_wallet_data;
    let authenticator = &ctx.accounts.smart_wallet_authenticator;

    // Load and verify the Secp256r1 instruction
    let instruction: Instruction = load_instruction_at_checked(
        args.verify_instruction_index as usize,
        &ctx.accounts.ix_sysvar,
    )?;

    verify_secp256r1_ix(
        &instruction,
        authenticator.passkey_pubkey,
        args.message,
        args.signature,
    )?;

    let wallet_signer = Some(crate::utils::PdaSigner {
        seeds: SMART_WALLET_SEED
            .iter()
            .chain(wallet_data.id.to_le_bytes().iter())
            .cloned()
            .collect(),
        bump: wallet_data.bump,
    });

    match wallet_data.rule_program {
        Some(rule_program_key) => {
            // Verify the rule program
            let rule_program = &ctx.accounts.cpi_program;
            require!(
                rule_program_key == rule_program.key(),
                LazorKitError::InvalidHook
            );

            // CPI to the rule program using remaining accounts
            execute_cpi_instruction(
                ctx.remaining_accounts,
                args.cpi_data.clone(),
                rule_program,
                wallet_signer.clone(),
            )?;
        }
        None => {
            // Directly execute the dApp instruction using remaining accounts
            execute_cpi_instruction(
                ctx.remaining_accounts,
                args.cpi_data,
                &ctx.accounts.cpi_program,
                wallet_signer,
            )?;
        }
    }

    Ok(())
}

#[derive(Accounts)]
pub struct ExecuteInstruction<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [SMART_WALLET_SEED, smart_wallet_data.id.to_le_bytes().as_ref()],
        bump,
        owner = ID,
    )]
    /// CHECK: This account is only used for its public key and seeds.
    pub smart_wallet: UncheckedAccount<'info>,

    #[account(
        seeds = [SmartWalletData::PREFIX_SEED, smart_wallet.key().as_ref()],
        bump,
        owner = ID,
    )]
    pub smart_wallet_data: Box<Account<'info, SmartWalletData>>,

    #[account(
        mut,
        seeds = [SmartWalletAuthenticator::PREFIX_SEED, smart_wallet.key().as_ref()],
        bump,
    )]
    pub smart_wallet_authenticator: Box<Account<'info, SmartWalletAuthenticator>>,

    #[account(
        seeds = [WhitelistRulePrograms::PREFIX_SEED],
        bump,
        owner = ID
    )]
    pub whitelist_rule_programs: Box<Account<'info, WhitelistRulePrograms>>,

    pub system_program: Program<'info, System>,

    /// CHECK: This account is used for CPI and is not deserialized.
    pub cpi_program: UncheckedAccount<'info>,

    #[account(address = IX_ID)]
    /// CHECK:
    pub ix_sysvar: UncheckedAccount<'info>,
}
