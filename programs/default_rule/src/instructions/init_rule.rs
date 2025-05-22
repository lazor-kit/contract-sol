use crate::error::RuleError;
use crate::state::{Config, Rule};
use crate::ID;
use anchor_lang::prelude::*;
use lazorkit::program::Lazorkit;

pub fn init_rule(ctx: Context<InitRule>) -> Result<()> {
    let rule = &mut ctx.accounts.rule;

    rule.smart_wallet = ctx.accounts.smart_wallet.key();
    rule.admin = ctx.accounts.smart_wallet_authenticator.key();
    rule.is_initialized = true;

    Ok(())
}

#[derive(Accounts)]
pub struct InitRule<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    pub lazorkit_authority: Signer<'info>,

    #[account(
        owner = ID,
        constraint = lazorkit_authority.key() == config.authority @ RuleError::UnAuthorize,
    )]
    pub config: Account<'info, Config>,

    /// CHECK:
    pub smart_wallet: UncheckedAccount<'info>,

    /// CHECK
    pub smart_wallet_authenticator: UncheckedAccount<'info>,

    #[account(
        init,
        payer = payer,
        space = 8 + Rule::INIT_SPACE,
        seeds = [b"rule".as_ref(), smart_wallet.key().as_ref()],
        bump,
    )]
    pub rule: Account<'info, Rule>,

    pub lazorkit: Program<'info, Lazorkit>,

    pub system_program: Program<'info, System>,
}
