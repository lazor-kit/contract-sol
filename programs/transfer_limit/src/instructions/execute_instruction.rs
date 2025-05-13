use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;
use lazorkit::{
    constants::SMART_WALLET_SEED,
    program::Lazorkit,
    state::{SmartWalletAuthenticator, SmartWalletData},
    utils::{execute_cpi, PasskeyExt},
};

use crate::{
    errors::TransferLimitError,
    state::{Member, MemberType, RuleData},
    utils::{close_account, get_token_account_and_balance},
    ID,
};

/// Arguments for the execute instruction
#[derive(Debug, AnchorDeserialize, AnchorSerialize)]
pub struct ExecuteInstructionArgs {
    /// Public key of the passkey used for authentication
    pub passkey_pubkey: [u8; 33],
    /// Optional token mint address. None for native SOL
    pub token: Option<Pubkey>,
    /// Serialized instruction data for CPI
    pub cpi_data: Vec<u8>,
}

pub fn execute_instruction<'c: 'info, 'info>(
    ctx: Context<'_, '_, 'c, 'info, ExecuteInstruction<'info>>,
    args: ExecuteInstructionArgs,
) -> Result<()> {
    let member = &ctx.accounts.member;
    let was_initialized = ctx.accounts.rule_data.is_initialized;

    // Handle member with initialized rule
    if member.member_type == MemberType::Member && was_initialized {
        // Validate token matches rule
        require!(
            ctx.accounts.rule_data.token == args.token,
            TransferLimitError::InvalidToken
        );

        // Get initial balance and prepare remaining accounts
        let (remaining_accounts, token_account, balance_before) = match args.token {
            Some(token) => {
                let (account, balance) = get_token_account_and_balance(
                    &ctx.accounts.smart_wallet.key(),
                    &token,
                    &mut ctx.remaining_accounts.iter(),
                )?;
                (&ctx.remaining_accounts[2..], Some(account), balance)
            }
            None => (
                ctx.remaining_accounts,
                None,
                ctx.accounts.smart_wallet.lamports(),
            ),
        };

        // Execute the transaction
        execute_cpi(
            remaining_accounts,
            args.cpi_data,
            &ctx.accounts.cpi_program,
            None,
        )?;

        // Get final balance and verify transfer amount
        let balance_after = match token_account {
            Some(token_account) => {
                InterfaceAccount::<TokenAccount>::try_from(token_account)?.amount
            }
            None => ctx.accounts.smart_wallet.lamports(),
        };

        require!(
            balance_before > balance_after,
            TransferLimitError::InvalidBalance
        );

        let transfer_amount = balance_before - balance_after;

        require!(
            transfer_amount > 0,
            TransferLimitError::InvalidTransferAmount
        );

        require!(
            transfer_amount <= ctx.accounts.rule_data.limit_amount,
            TransferLimitError::InvalidTransferAmount
        );
    } else {
        // Execute CPI first
        execute_cpi(
            ctx.remaining_accounts,
            args.cpi_data,
            &ctx.accounts.cpi_program,
            None,
        )?;

        // If rule was newly created but not used, close it
        if !was_initialized {
            close_account(
                &ctx.accounts.rule_data.to_account_info(),
                &ctx.accounts.signer.to_account_info(),
            );
        }
    }

    Ok(())
}

/// Accounts required for the execute instruction
#[derive(Accounts)]
#[instruction(args: ExecuteInstructionArgs)]
pub struct ExecuteInstruction<'info> {
    /// Signer of the transaction, pays for account creation
    #[account(mut)]
    pub signer: Signer<'info>,

    /// Smart wallet PDA that can execute transactions
    #[account(
        seeds = [SMART_WALLET_SEED, smart_wallet_data.id.to_le_bytes().as_ref()],
        bump,
        seeds::program = lazorkit.key(),
        signer
    )]
    /// CHECK: Validated by seeds
    pub smart_wallet: UncheckedAccount<'info>,

    /// Member account that authorizes the transaction
    #[account(
        seeds = [Member::PREFIX_SEED, smart_wallet.key().as_ref(), smart_wallet_authenticator.key().as_ref()],
        bump,
        owner = ID,
        constraint = member.owner == smart_wallet_authenticator.key(),
    )]
    pub member: Account<'info, Member>,

    /// Rule data account that enforces transfer limits
    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + RuleData::INIT_SPACE,
        seeds = [RuleData::PREFIX_SEED, smart_wallet.key().as_ref(), args.token.as_ref().unwrap_or(&Pubkey::default()).as_ref()],
        bump,
    )]
    pub rule_data: Account<'info, RuleData>,

    /// Smart wallet data account storing configuration
    #[account(
        seeds  = [SmartWalletData::PREFIX_SEED, smart_wallet.key().as_ref()],
        bump,
        seeds::program = lazorkit.key(),
    )]
    pub smart_wallet_data: Account<'info, SmartWalletData>,

    /// Authenticator account for passkey verification
    #[account(
        seeds = [args.passkey_pubkey.to_hashed_bytes(smart_wallet.key()).as_ref()],
        bump,
        seeds::program = lazorkit.key(),
    )]
    pub smart_wallet_authenticator: Account<'info, SmartWalletAuthenticator>,

    /// Program to execute CPI to
    /// CHECK: Validated in CPI
    pub cpi_program: UncheckedAccount<'info>,

    /// Lazorkit program for cross-program invocation
    pub lazorkit: Program<'info, Lazorkit>,

    /// System program for account creation
    pub system_program: Program<'info, System>,
}
