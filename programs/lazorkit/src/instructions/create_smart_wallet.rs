use anchor_lang::prelude::*;

use crate::{
    constants::{PASSKEY_SIZE, SMART_WALLET_SEED},
    state::{Config, SmartWalletAuthenticator, SmartWalletConfig, SmartWalletSeq},
    utils::{transfer_sol_from_pda, PasskeyExt},
    ID,
};

pub fn create_smart_wallet(
    ctx: Context<CreateSmartWallet>,
    passkey_pubkey: [u8; PASSKEY_SIZE],
    credential_id: Vec<u8>,
) -> Result<()> {
    let wallet_data = &mut ctx.accounts.smart_wallet_config;
    let sequence_account = &mut ctx.accounts.smart_wallet_seq;
    let smart_wallet_authenticator = &mut ctx.accounts.smart_wallet_authenticator;

    wallet_data.set_inner(SmartWalletConfig {
        id: sequence_account.seq,
        last_nonce: 0,
        bump: ctx.bumps.smart_wallet,
    });

    // Initialize the smart wallet authenticator
    smart_wallet_authenticator.set_inner(SmartWalletAuthenticator {
        passkey_pubkey,
        smart_wallet: ctx.accounts.smart_wallet.key(),
        credential_id,
        bump: ctx.bumps.smart_wallet_authenticator,
    });

    sequence_account.seq += 1;

    transfer_sol_from_pda(
        &ctx.accounts.smart_wallet,
        &mut ctx.accounts.signer,
        ctx.accounts.config.create_smart_wallet_fee,
    )?;

    Ok(())
}

#[derive(Accounts)]
#[instruction(passkey_pubkey: [u8; PASSKEY_SIZE])]
pub struct CreateSmartWallet<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    /// CHECK: This account is only used for its public key and seeds.
    #[account(
        mut,
        seeds = [SmartWalletSeq::PREFIX_SEED],
        bump,
    )]
    pub smart_wallet_seq: Account<'info, SmartWalletSeq>,

    #[account(
        init,
        payer = signer,
        space = 0,
        seeds = [SMART_WALLET_SEED, smart_wallet_seq.seq.to_le_bytes().as_ref()],
        bump
    )]
    /// CHECK: This account is only used for its public key and seeds.
    pub smart_wallet: UncheckedAccount<'info>,

    #[account(
        init,
        payer = signer,
        space = 8 + SmartWalletConfig::INIT_SPACE,
        seeds = [SmartWalletConfig::PREFIX_SEED, smart_wallet.key().as_ref()],
        bump
    )]
    pub smart_wallet_config: Box<Account<'info, SmartWalletConfig>>,

    #[account(
        init,
        payer = signer,
        space = 8 + SmartWalletAuthenticator::INIT_SPACE,
        seeds = [
            SmartWalletAuthenticator::PREFIX_SEED,
            smart_wallet.key().as_ref(),
            passkey_pubkey.to_hashed_bytes(smart_wallet.key()).as_ref()
        ],
        bump
    )]
    pub smart_wallet_authenticator: Box<Account<'info, SmartWalletAuthenticator>>,

    #[account(
        seeds = [Config::PREFIX_SEED],
        bump,
        owner = ID
    )]
    pub config: Box<Account<'info, Config>>,

    pub system_program: Program<'info, System>,
}
