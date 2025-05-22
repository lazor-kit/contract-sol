use anchor_lang::prelude::*;
use anchor_spl::token_interface::TokenAccount;
use lazorkit::utils::transfer_sol_from_pda;
use lazorkit::{
    constants::SMART_WALLET_SEED,
    program::Lazorkit,
    state::{SmartWalletAuthenticator, SmartWalletConfig},
    utils::{execute_cpi, PasskeyExt, PdaSigner},
};

use crate::{
    errors::TransferLimitError,
    state::{Member, MemberType, RuleData},
    utils::{close_account, get_token_account_and_balance},
    ID,
};

/// Arguments for the execute instruction
#[derive(Debug, AnchorDeserialize, AnchorSerialize, Clone)]
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
    let smart_wallet_config = &ctx.accounts.smart_wallet_config;

    // Handle SOL transfer
    if ctx.accounts.cpi_program.key() == anchor_lang::solana_program::system_program::ID {
        return handle_sol_transfer(&ctx, &args, member, was_initialized);
    }

    // Handle other CPIs
    handle_cpi(&ctx, &args, member, was_initialized, smart_wallet_config)
}

fn handle_sol_transfer<'info>(
    ctx: &Context<'_, '_, '_, 'info, ExecuteInstruction<'info>>,
    args: &ExecuteInstructionArgs,
    member: &Account<Member>,
    was_initialized: bool,
) -> Result<()> {
    require!(
        ctx.remaining_accounts.len() >= 2,
        TransferLimitError::InvalidAccountInput
    );

    let amount = u64::from_le_bytes(args.cpi_data[4..12].try_into().unwrap());

    // Check transfer limit for non-admin members
    if member.member_type == MemberType::Member && was_initialized {
        require!(
            amount <= ctx.accounts.rule_data.limit_amount,
            TransferLimitError::InvalidTransferAmount
        );
    }

    transfer_sol_from_pda(
        &ctx.accounts.smart_wallet,
        &ctx.remaining_accounts[1].to_account_info(),
        amount,
    )?;

    Ok(())
}

fn handle_cpi<'info>(
    ctx: &Context<'_, '_, 'info, 'info, ExecuteInstruction<'info>>,
    args: &ExecuteInstructionArgs,
    member: &Account<Member>,
    was_initialized: bool,
    smart_wallet_config: &Account<SmartWalletConfig>,
) -> Result<()> {
    if member.member_type == MemberType::Member && was_initialized {
        validate_cpi(ctx, args)?;
    } else {
        execute_cpi_with_signer(ctx, args, smart_wallet_config)?;

        // Close rule if newly created but not used
        if !was_initialized {
            close_account(
                &ctx.accounts.rule_data.to_account_info(),
                &ctx.accounts.smart_wallet.to_account_info(),
            );
        }
    }

    Ok(())
}

fn validate_cpi<'info>(
    ctx: &Context<'_, '_, 'info, 'info, ExecuteInstruction<'info>>,
    args: &ExecuteInstructionArgs,
) -> Result<()> {
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
        args.cpi_data.clone(),
        &ctx.accounts.cpi_program,
        None,
    )?;

    // Get final balance and verify transfer amount
    let balance_after = match token_account {
        Some(token_account) => InterfaceAccount::<TokenAccount>::try_from(token_account)?.amount,
        None => ctx.accounts.smart_wallet.lamports(),
    };

    require!(
        balance_before > balance_after,
        TransferLimitError::InvalidBalance
    );

    let transfer_amount = balance_before - balance_after;
    require!(
        transfer_amount > 0 && transfer_amount <= ctx.accounts.rule_data.limit_amount,
        TransferLimitError::InvalidTransferAmount
    );

    Ok(())
}

fn execute_cpi_with_signer<'info>(
    ctx: &Context<'_, '_, '_, 'info, ExecuteInstruction<'info>>,
    args: &ExecuteInstructionArgs,
    smart_wallet_config: &Account<SmartWalletConfig>,
) -> Result<()> {
    let smart_wallet_signer = [SMART_WALLET_SEED, &smart_wallet_config.id.to_le_bytes()].concat();

    execute_cpi(
        ctx.remaining_accounts,
        args.cpi_data.clone(),
        &ctx.accounts.cpi_program,
        Some(PdaSigner {
            seeds: smart_wallet_signer,
            bump: smart_wallet_config.bump,
        }),
    )
}

/// Accounts required for the execute instruction
#[derive(Accounts)]
#[instruction(args: ExecuteInstructionArgs)]
pub struct ExecuteInstruction<'info> {
    /// Smart wallet PDA that can execute transactions
    #[account(mut)]
    pub smart_wallet: Signer<'info>,

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
        payer = smart_wallet,
        space = 8 + RuleData::INIT_SPACE,
        seeds = [RuleData::PREFIX_SEED, smart_wallet.key().as_ref(), args.token.as_ref().unwrap_or(&Pubkey::default()).as_ref()],
        bump,
    )]
    pub rule_data: Account<'info, RuleData>,

    /// Smart wallet data account storing configuration
    #[account(
        seeds  = [SmartWalletConfig::PREFIX_SEED, smart_wallet.key().as_ref()],
        bump,
        seeds::program = lazorkit.key(),
    )]
    pub smart_wallet_config: Account<'info, SmartWalletConfig>,

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
