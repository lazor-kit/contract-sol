use anchor_lang::prelude::*;
use lazorkit::{program::Lazorkit, state::SmartWalletAuthenticator, utils::PasskeyExt};

use crate::{
    errors::TransferLimitError,
    state::{Member, MemberType},
    ID,
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct AddMemberArgs {
    pub member: Pubkey,
}

pub fn add_member(ctx: Context<AddMember>, new_passkey_pubkey: [u8; 33]) -> Result<()> {
    let member = &mut ctx.accounts.member;
    let new_smart_wallet_authenticator = &mut ctx.accounts.new_smart_wallet_authenticator;
    let smart_wallet_authenticator = &mut ctx.accounts.smart_wallet_authenticator;

    let seeds: &[&[u8]] =
        &[&new_passkey_pubkey.to_hashed_bytes(smart_wallet_authenticator.smart_wallet.key())];
    let (expected_pubkey, expected_bump) =
        Pubkey::find_program_address(seeds, &ctx.accounts.lazorkit.key());

    require!(
        expected_pubkey == new_smart_wallet_authenticator.key(),
        TransferLimitError::InvalidNewPasskey
    );

    member.set_inner(Member {
        owner: new_smart_wallet_authenticator.key(),
        member_type: MemberType::Member,
        smart_wallet: smart_wallet_authenticator.smart_wallet,
        bump: expected_bump,
        is_initialized: true,
    });

    new_smart_wallet_authenticator.set_inner(SmartWalletAuthenticator {
        passkey_pubkey: new_passkey_pubkey,
        smart_wallet: smart_wallet_authenticator.smart_wallet,
        bump: expected_bump,
    });

    Ok(())
}

#[derive(Accounts)]
pub struct AddMember<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        owner = lazorkit.key()
    )]
    pub smart_wallet_authenticator: Account<'info, SmartWalletAuthenticator>,

    #[account(
        init,
        payer = payer,
        space = 8 + SmartWalletAuthenticator::INIT_SPACE,
        owner = lazorkit.key()
    )]
    /// CHECK:
    pub new_smart_wallet_authenticator: Box<Account<'info, SmartWalletAuthenticator>>,

    #[account(
        seeds = [Member::PREFIX_SEED, smart_wallet_authenticator.smart_wallet.key().as_ref(), smart_wallet_authenticator.key().as_ref()],
        bump,
        owner = ID,
        constraint = admin.member_type == MemberType::Admin,
    )]
    pub admin: Account<'info, Member>,

    #[account(
        init,
        payer = payer,
        space = 8 + Member::INIT_SPACE,
        seeds = [Member::PREFIX_SEED, smart_wallet_authenticator.smart_wallet.key().as_ref(), new_smart_wallet_authenticator.key().as_ref()],
        bump,
    )]
    pub member: Account<'info, Member>,

    pub lazorkit: Program<'info, Lazorkit>,

    pub system_program: Program<'info, System>,
}
