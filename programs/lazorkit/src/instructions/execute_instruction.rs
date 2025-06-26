use anchor_lang::solana_program::hash::hash; // ✅ required import

use anchor_lang::{prelude::*, solana_program::sysvar::instructions::load_instruction_at_checked};

use crate::state::{Config, Message};
use crate::utils::{
    execute_cpi, transfer_sol_from_pda, verify_secp256r1_instruction, PasskeyExt, PdaSigner,
};
use crate::{
    constants::{SMART_WALLET_SEED, SOL_TRANSFER_DISCRIMINATOR},
    error::LazorKitError,
    state::{SmartWalletAuthenticator, SmartWalletConfig},
    ID,
};
use anchor_lang::solana_program::sysvar::instructions::ID as IX_ID;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};

const MAX_TIMESTAMP_DRIFT_SECONDS: i64 = 30;

/// Arguments for the execute_instruction entrypoint
#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct ExecuteInstructionArgs {
    pub passkey_pubkey: [u8; 33],
    pub signature: Vec<u8>,
    pub client_data_json_raw: Vec<u8>, // Match field name used in SDK
    pub authenticator_data_raw: Vec<u8>, // Added missing field
    pub verify_instruction_index: u8,
    pub cpi_data: Vec<u8>,
}

/// Entrypoint for executing smart wallet instructions
pub fn execute_instruction(
    mut ctx: Context<ExecuteInstruction>,
    args: ExecuteInstructionArgs,
) -> Result<()> {
    verify_passkey_and_signature(&ctx, &args, ctx.accounts.smart_wallet_config.last_nonce)?;

    handle_execute_cpi(&mut ctx, &args)?;

    // update last nonce
    ctx.accounts.smart_wallet_config.last_nonce = ctx
        .accounts
        .smart_wallet_config
        .last_nonce
        .checked_add(1)
        .ok_or(LazorKitError::NonceOverflow)?;

    Ok(())
}

fn verify_passkey_and_signature(
    ctx: &Context<ExecuteInstruction>,
    args: &ExecuteInstructionArgs,
    last_nonce: u64,
) -> Result<()> {
    let authenticator = &ctx.accounts.smart_wallet_authenticator;

    // --- Passkey validation ---
    require!(
        authenticator.passkey_pubkey == args.passkey_pubkey,
        LazorKitError::PasskeyMismatch
    );

    // --- Smart wallet validation ---
    require!(
        authenticator.smart_wallet == ctx.accounts.smart_wallet.key(),
        LazorKitError::SmartWalletMismatch
    );

    // --- Signature verification using secp256r1 ---
    let secp_ix = load_instruction_at_checked(
        args.verify_instruction_index as usize,
        &ctx.accounts.ix_sysvar,
    )?;

    let client_hash = hash(&args.client_data_json_raw);

    let mut message =
        Vec::with_capacity(args.authenticator_data_raw.len() + client_hash.as_ref().len());
    message.extend_from_slice(&args.authenticator_data_raw);
    message.extend_from_slice(client_hash.as_ref());

    let json_str = core::str::from_utf8(&args.client_data_json_raw)
        .map_err(|_| LazorKitError::ClientDataInvalidUtf8)?;
    let parsed: serde_json::Value =
        serde_json::from_str(json_str).map_err(|_| LazorKitError::ClientDataJsonParseError)?;
    let challenge = parsed["challenge"]
        .as_str()
        .ok_or(LazorKitError::ChallengeMissing)?;

    // Remove surrounding quotes, slashes and whitespace from challenge
    let challenge_clean = challenge.trim_matches(|c| c == '"' || c == '\'' || c == '/' || c == ' ');
    let challenge_bytes = URL_SAFE_NO_PAD
        .decode(challenge_clean)
        .map_err(|_| LazorKitError::ChallengeBase64DecodeError)?;

    let msg = Message::try_from_slice(&challenge_bytes)
        .map_err(|_| LazorKitError::ChallengeDeserializationError)?;

    let now = Clock::get()?.unix_timestamp;

    // check if timestamp is within the allowed drift
    if msg.timestamp < now.saturating_sub(MAX_TIMESTAMP_DRIFT_SECONDS) {
        return Err(LazorKitError::TimestampTooOld.into());
    }
    if msg.timestamp > now.saturating_add(MAX_TIMESTAMP_DRIFT_SECONDS) {
        return Err(LazorKitError::TimestampTooNew.into());
    }

    // check if nonce matches the expected nonce
    require!(msg.nonce == last_nonce, LazorKitError::NonceMismatch);

    verify_secp256r1_instruction(
        &secp_ix,
        authenticator.passkey_pubkey,
        message,
        args.signature.clone(),
    )
}

fn handle_execute_cpi(
    ctx: &mut Context<ExecuteInstruction>,
    args: &ExecuteInstructionArgs,
) -> Result<()> {
    let cpi_data = &args.cpi_data;

    // --- Special handling for SOL transfer ---
    if cpi_data.get(0..4) == Some(&SOL_TRANSFER_DISCRIMINATOR)
        && ctx.accounts.cpi_program.key() == anchor_lang::solana_program::system_program::ID
    {
        require!(
            ctx.remaining_accounts.len() >= 2,
            LazorKitError::SolTransferInsufficientAccounts
        );
        let amount = u64::from_le_bytes(cpi_data[4..12].try_into().unwrap());
        transfer_sol_from_pda(
            &ctx.accounts.smart_wallet,
            &ctx.remaining_accounts[1].to_account_info(),
            amount,
        )?;
    } else {
        // --- Generic CPI with wallet signer ---
        let wallet_signer = PdaSigner {
            seeds: vec![
                SMART_WALLET_SEED.to_vec(),
                ctx.accounts.smart_wallet_config.id.to_le_bytes().to_vec(),
            ],
            bump: ctx.accounts.smart_wallet_config.bump,
        };
        execute_cpi(
            ctx.remaining_accounts,
            &cpi_data,
            &ctx.accounts.cpi_program,
            Some(wallet_signer),
        )?;
    }
    Ok(())
}

/// Accounts context for execute_instruction
#[derive(Accounts)]
#[instruction(args: ExecuteInstructionArgs)]
pub struct ExecuteInstruction<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [Config::PREFIX_SEED],
        bump,
        owner = ID
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(
        mut,
        seeds = [SMART_WALLET_SEED, smart_wallet_config.id.to_le_bytes().as_ref()],
        bump,
        owner = ID,
    )]
    /// CHECK: Only used for key and seeds.
    pub smart_wallet: UncheckedAccount<'info>,

    #[account(
        mut,
        seeds = [SmartWalletConfig::PREFIX_SEED, smart_wallet.key().as_ref()],
        bump,
        owner = ID,
    )]
    pub smart_wallet_config: Box<Account<'info, SmartWalletConfig>>,

    #[account(
        seeds = [
            SmartWalletAuthenticator::PREFIX_SEED,
            smart_wallet.key().as_ref(),
            args.passkey_pubkey.to_hashed_bytes(smart_wallet.key()).as_ref()
        ],
        bump,
        owner = ID,
    )]
    pub smart_wallet_authenticator: Box<Account<'info, SmartWalletAuthenticator>>,

    #[account(address = IX_ID)]
    /// CHECK: Sysvar for instructions.
    pub ix_sysvar: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: Used for CPI, not deserialized.
    pub cpi_program: UncheckedAccount<'info>,
}
