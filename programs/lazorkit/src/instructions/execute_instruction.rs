use anchor_lang::{prelude::*, solana_program::sysvar::instructions::load_instruction_at_checked};

use crate::{
    constants::SMART_WALLET_SEED,
    error::LazorKitError,
    state::{SmartWalletAuthenticator, SmartWalletData, WhitelistRulePrograms},
    utils::{
        execute_cpi, transfer_sol_from_pda, verify_secp256r1_instruction, PasskeyExt, PdaSigner,
    },
    ID,
};
use anchor_lang::solana_program::sysvar::instructions::ID as IX_ID;

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExecuteInstructionArgs {
    pub passkey_pubkey: [u8; 33],
    pub signature: Vec<u8>,
    pub message: Vec<u8>,
    pub verify_instruction_index: u8,
    pub cpi_data: CpiData,
    pub rule_data: CpiData,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CpiData {
    pub data: Vec<u8>,
    pub start_index: u8, // starting index in remaining accounts
    pub length: u8,      // number of accounts to take from remaining accounts
}

pub fn execute_instruction(
    ctx: Context<ExecuteInstruction>,
    args: ExecuteInstructionArgs,
) -> Result<()> {
    let smart_wallet_auth = &ctx.accounts.smart_wallet_authenticator;
    let payer = &ctx.accounts.payer;
    let payer_balance_before = payer.lamports();

    // Verify passkey and smart wallet association
    require!(
        smart_wallet_auth.passkey_pubkey == args.passkey_pubkey
            && smart_wallet_auth.smart_wallet == ctx.accounts.smart_wallet.key(),
        LazorKitError::InvalidPasskey
    );

    // Load and verify the Secp256r1 instruction
    let secp_ix = load_instruction_at_checked(
        args.verify_instruction_index as usize,
        &ctx.accounts.ix_sysvar,
    )?;

    verify_secp256r1_instruction(
        &secp_ix,
        smart_wallet_auth.passkey_pubkey,
        args.message,
        args.signature,
    )?;

    // Check if the rule program is whitelisted
    let rule_program_key = ctx.accounts.rule_program.key();
    let whitelist = &ctx.accounts.whitelist_rule_programs;
    require!(
        whitelist.list.contains(&rule_program_key),
        LazorKitError::InvalidRuleProgram
    );

    // Prepare PDA signer for rule CPI
    let auth_signer = PdaSigner {
        seeds: args
            .passkey_pubkey
            .to_hashed_bytes(ctx.accounts.smart_wallet.key())
            .to_vec(),
        bump: ctx.bumps.smart_wallet_authenticator,
    };

    // Slice rule accounts from remaining_accounts
    let rule_accounts = ctx
        .remaining_accounts
        .get(
            args.rule_data.start_index as usize
                ..(args.rule_data.start_index as usize + args.rule_data.length as usize),
        )
        .ok_or(LazorKitError::InvalidAccountInput)?;

    execute_cpi(
        rule_accounts,
        args.rule_data.data,
        &ctx.accounts.rule_program,
        Some(auth_signer),
    )?;

    // Slice CPI accounts from remaining_accounts
    let cpi_accounts = ctx
        .remaining_accounts
        .get(
            args.cpi_data.start_index as usize
                ..(args.cpi_data.start_index as usize + args.cpi_data.length as usize),
        )
        .ok_or(LazorKitError::InvalidAccountInput)?;

    // Handle SOL transfer or generic CPI
    if ctx.accounts.cpi_program.key() == anchor_lang::solana_program::system_program::ID {
        require!(
            ctx.remaining_accounts.len() >= 2,
            LazorKitError::InvalidAccountInput
        );
        let amount = u64::from_le_bytes(args.cpi_data.data[4..12].try_into().unwrap());
        transfer_sol_from_pda(
            &ctx.accounts.smart_wallet,
            &ctx.remaining_accounts[1].to_account_info(),
            amount,
        )?;
    } else {
        let wallet_data = &ctx.accounts.smart_wallet_data;
        let wallet_signer = PdaSigner {
            seeds: [SMART_WALLET_SEED, wallet_data.id.to_le_bytes().as_ref()].concat(),
            bump: wallet_data.bump,
        };
        execute_cpi(
            cpi_accounts,
            args.cpi_data.data,
            &ctx.accounts.cpi_program,
            Some(wallet_signer),
        )?;
    }

    // Reimburse payer if balance changed
    let payer_balance_after = payer.lamports().saturating_sub(10000);
    let reimbursement = payer_balance_before.saturating_sub(payer_balance_after);
    if reimbursement > 0 {
        transfer_sol_from_pda(
            &ctx.accounts.smart_wallet,
            &ctx.accounts.payer,
            reimbursement,
        )?;
    }

    Ok(())
}

#[derive(Accounts)]
#[instruction(args: ExecuteInstructionArgs)]
pub struct ExecuteInstruction<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [SMART_WALLET_SEED, smart_wallet_data.id.to_le_bytes().as_ref()],
        bump,
        owner = ID,
    )]
    /// CHECK: Only used for key and seeds.
    pub smart_wallet: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [SmartWalletData::PREFIX_SEED, smart_wallet.key().as_ref()],
        bump,
        owner = ID,
    )]
    pub smart_wallet_data: Account<'info, SmartWalletData>,

    #[account(
        seeds = [args.passkey_pubkey.to_hashed_bytes(smart_wallet.key()).as_ref()],
        bump,
    )]
    pub smart_wallet_authenticator: Account<'info, SmartWalletAuthenticator>,

    #[account(
        seeds = [WhitelistRulePrograms::PREFIX_SEED],
        bump,
        owner = ID
    )]
    pub whitelist_rule_programs: Box<Account<'info, WhitelistRulePrograms>>,

    /// CHECK: Used for CPI, not deserialized.
    pub cpi_program: UncheckedAccount<'info>,

    /// CHECK: Used for rule CPI.
    pub rule_program: UncheckedAccount<'info>,

    #[account(address = IX_ID)]
    /// CHECK: Sysvar for instructions.
    pub ix_sysvar: UncheckedAccount<'info>,
}
