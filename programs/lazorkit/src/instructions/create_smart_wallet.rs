use anchor_lang::prelude::*;

use crate::{
    constants::SMART_WALLET_SEED,
    error::LazorKitError,
    state::{SmartWalletAuthenticator, SmartWalletData, SmartWalletSeq, WhitelistRulePrograms},
    utils::execute_cpi_instruction,
    ID,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct CreateSmartWalletArgs {
    pub passkey_pubkey: [u8; 33],
    pub rule_data: Option<Vec<u8>>,
}

pub fn create_smart_wallet(
    ctx: Context<CreateSmartWallet>,
    args: CreateSmartWalletArgs,
) -> Result<()> {
    let wallet_data = &mut ctx.accounts.smart_wallet_data;
    let sequence_account = &mut ctx.accounts.smart_wallet_seq;

    if let Some(rule_data) = args.rule_data {
        let whitelist_rule_programs = &ctx.accounts.whitelist_rule_programs;
        let remaining_accounts = ctx.remaining_accounts;
        let rule_program = &ctx.accounts.rule_program;

        // Ensure the rule program is whitelisted
        require!(
            whitelist_rule_programs.list.contains(&rule_program.key()),
            LazorKitError::InvalidRuleProgram
        );

        // Execute the CPI instruction
        execute_cpi_instruction(
            &remaining_accounts,
            rule_data,
            &rule_program.to_account_info(),
            None,
        )?;

        wallet_data.set_inner(SmartWalletData {
            rule_program: Some(ctx.accounts.rule_program.key()),
            id: sequence_account.seq,
            bump: ctx.bumps.smart_wallet,
        });
    } else {
        wallet_data.set_inner(SmartWalletData {
            rule_program: None,
            id: sequence_account.seq,
            bump: ctx.bumps.smart_wallet,
        });
    }

    sequence_account.seq += 1;
    Ok(())
}

#[derive(Accounts)]
pub struct CreateSmartWallet<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [SmartWalletSeq::PREFIX_SEED],
        bump,
    )]
    pub smart_wallet_seq: Box<Account<'info, SmartWalletSeq>>,

    #[account(
        seeds = [WhitelistRulePrograms::PREFIX_SEED],
        bump,
        owner = ID
    )]
    pub whitelist_rule_programs: Box<Account<'info, WhitelistRulePrograms>>,

    #[account(
        init_if_needed,
        payer = signer,
        space = 0,
        seeds = [SMART_WALLET_SEED, smart_wallet_seq.seq.to_le_bytes().as_ref()],
        bump
    )]
    /// CHECK: This account is only used for its public key and seeds.
    pub smart_wallet: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + SmartWalletData::INIT_SPACE,
        seeds = [SmartWalletData::PREFIX_SEED, smart_wallet.key().as_ref()],
        bump
    )]
    pub smart_wallet_data: Box<Account<'info, SmartWalletData>>,

    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + SmartWalletAuthenticator::INIT_SPACE,
        seeds = [SmartWalletAuthenticator::PREFIX_SEED, smart_wallet.key().as_ref()],
        bump
    )]
    pub smart_wallet_authenticator: Box<Account<'info, SmartWalletAuthenticator>>,

    /// CHECK: This account is used for CPI and is not deserialized.
    pub rule_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
