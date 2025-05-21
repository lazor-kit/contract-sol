use crate::error::RuleError;
use crate::state::{Config, Rule};
use crate::ID;
use anchor_lang::prelude::*;
use lazorkit::{program::Lazorkit, state::SmartWalletAuthenticator, utils::PasskeyExt};

pub fn init_rule(ctx: Context<CreateRule>, passkey_pubkey: [u8; 33]) -> Result<()> {
    let rule = &mut ctx.accounts.rule;

    // check rule is created or not
    if rule.is_initialized {
        // need to check that admin of smart-wallet
        
    } else {
    }
    Ok(())
}

#[derive(Accounts)]
#[instruction(passkey_pubkey: [u8; 33])]
pub struct CreateRule<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(mut)]
    pub author: Signer<'info>,

    #[account(
        owner = ID
    )]
    pub config: Account<'info, Config>,

    /// CHECK:
    pub smart_wallet: UncheckedAccount<'info>,

    #[account(
        init_if_needed,
        payer = payer,
        space = 8 + Rule::INIT_SPACE,
        seeds = [b"rule".as_ref(), smart_wallet.key().as_ref()],
        bump,
    )]
    pub rule: Account<'info, Rule>,

    pub lazorkit: Program<'info, Lazorkit>,

    pub system_program: Program<'info, System>,
}
