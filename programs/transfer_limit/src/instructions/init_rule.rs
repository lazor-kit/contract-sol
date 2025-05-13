use anchor_lang::prelude::*;
use lazorkit::{
    constants::SMART_WALLET_SEED,
    program::Lazorkit,
    state::{SmartWalletAuthenticator, SmartWalletData},
    utils::PasskeyExt,
};

use crate::{state::*, ID};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct InitRuleArgs {
    pub passkey_pubkey: [u8; 33],
    pub token: Option<Pubkey>,
    pub limit_amount: u64,
    pub limit_period: u64,
}

pub fn init_rule(ctx: Context<InitRule>, args: InitRuleArgs) -> Result<()> {
    let smart_wallet_data = &mut ctx.accounts.smart_wallet_data;

    smart_wallet_data.rule_program = Some(ID);

    let rule_data = &mut ctx.accounts.rule_data;
    rule_data.set_inner(RuleData {
        token: args.token,
        limit_amount: args.limit_amount,
        bump: ctx.bumps.smart_wallet_authenticator,
        is_initialized: true,
    });

    let member = &mut ctx.accounts.member;
    if !member.is_initialized {
        member.set_inner(Member {
            smart_wallet: ctx.accounts.smart_wallet.key(),
            owner: ctx.accounts.smart_wallet_authenticator.key(),
            bump: ctx.bumps.smart_wallet_authenticator,
            is_initialized: true,
            member_type: MemberType::Admin,
        });
    }
    Ok(())
}

#[derive(Accounts)]
#[instruction(args: InitRuleArgs)]
pub struct InitRule<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        seeds = [SMART_WALLET_SEED, smart_wallet_data.id.to_le_bytes().as_ref()],
        bump,
        seeds::program = lazorkit.key(), // LazorKit ID
    )]
    /// CHECK
    pub smart_wallet: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + Member::INIT_SPACE,
        seeds = [Member::PREFIX_SEED, smart_wallet.key().as_ref(), smart_wallet_authenticator.key().as_ref()],
        bump,
    )]
    pub member: Box<Account<'info, Member>>,

    #[account(
        init,
        payer = payer,
        space = 8 + RuleData::INIT_SPACE,
        seeds = [RuleData::PREFIX_SEED, smart_wallet.key().as_ref(), args.token.as_ref().unwrap_or(&Pubkey::default()).as_ref()],
        bump,
    )]
    pub rule_data: Box<Account<'info, RuleData>>,

    #[account(
        mut,
        seeds  = [SmartWalletData::PREFIX_SEED, smart_wallet.key().as_ref()],
        bump,
        seeds::program = lazorkit.key(), // LazorKit ID
    )]
    pub smart_wallet_data: Account<'info, SmartWalletData>,

    #[account(
        seeds = [args.passkey_pubkey.to_hashed_bytes(smart_wallet.key()).as_ref()],
        bump,
        seeds::program = lazorkit.key(), // LazorKit ID
    )]
    pub smart_wallet_authenticator: Account<'info, SmartWalletAuthenticator>,

    pub lazorkit: Program<'info, Lazorkit>,

    pub system_program: Program<'info, System>,
}
