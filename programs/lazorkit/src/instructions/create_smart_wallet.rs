use anchor_lang::prelude::*;

use crate::{
    constants::{PASSKEY_SIZE, SMART_WALLET_SEED},
    state::{SmartWalletAuthenticator, SmartWalletData, SmartWalletSeq, WhitelistRulePrograms},
    utils::PasskeyExt,
    ID,
};

pub fn create_smart_wallet(
    ctx: Context<CreateSmartWallet>,
    passkey_pubkey: [u8; PASSKEY_SIZE],
) -> Result<()> {
    let wallet_data = &mut ctx.accounts.smart_wallet_data;
    let sequence_account = &mut ctx.accounts.smart_wallet_seq;
    let smart_wallet_authenticator = &mut ctx.accounts.smart_wallet_authenticator;

    wallet_data.set_inner(SmartWalletData {
        rule_program: None,
        id: sequence_account.seq,
        bump: ctx.bumps.smart_wallet,
    });

    // Initialize the smart wallet authenticator
    smart_wallet_authenticator.set_inner(SmartWalletAuthenticator {
        passkey_pubkey,
        smart_wallet: ctx.accounts.smart_wallet.key(),
        bump: ctx.bumps.smart_wallet_authenticator,
    });

    sequence_account.seq += 1;

    Ok(())
}

#[derive(Accounts)]
#[instruction(passkey_pubkey: [u8; PASSKEY_SIZE])]
pub struct CreateSmartWallet<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        mut,
        seeds = [SmartWalletSeq::PREFIX_SEED],
        bump,
    )]
    pub smart_wallet_seq: Account<'info, SmartWalletSeq>,

    #[account(
        seeds = [WhitelistRulePrograms::PREFIX_SEED],
        bump,
        owner = ID
    )]
    pub whitelist_rule_programs: Account<'info, WhitelistRulePrograms>,

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
        space = 8 + SmartWalletData::INIT_SPACE,
        seeds = [SmartWalletData::PREFIX_SEED, smart_wallet.key().as_ref()],
        bump
    )]
    pub smart_wallet_data: Box<Account<'info, SmartWalletData>>,

    #[account(
        init,
        payer = signer,
        space = 8 + SmartWalletAuthenticator::INIT_SPACE,
        seeds = [passkey_pubkey.to_hashed_bytes(smart_wallet.key()).as_ref()],
        bump
    )]
    pub smart_wallet_authenticator: Box<Account<'info, SmartWalletAuthenticator>>,

    pub system_program: Program<'info, System>,
}
