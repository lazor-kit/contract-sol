use anchor_lang::prelude::*;

use crate::state::{Config, SmartWalletSeq};

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let smart_wallet_seq = &mut ctx.accounts.smart_wallet_seq;
    smart_wallet_seq.seq = 0;

    let config: &mut Box<Account<'_, Config>> = &mut ctx.accounts.config;
    config.authority = ctx.accounts.signer.key();
    config.create_smart_wallet_fee = 0; // LAMPORTS
    config.execute_instruction_fee = 0; // LAMPORTS
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + Config::INIT_SPACE,
        seeds = [Config::PREFIX_SEED],
        bump,
    )]
    pub config: Box<Account<'info, Config>>,

    #[account(
        init_if_needed,
        payer = signer,
        space = 8 + SmartWalletSeq::INIT_SPACE,
        seeds = [SmartWalletSeq::PREFIX_SEED],
        bump
    )]
    pub smart_wallet_seq: Box<Account<'info, SmartWalletSeq>>,

    /// CHECK:
    pub default_rule_program: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}
