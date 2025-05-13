use anchor_lang::prelude::*;

use crate::state::{SmartWalletSeq, WhitelistRulePrograms};

pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
    let whitelist_rule_programs = &mut ctx.accounts.whitelist_rule_programs;
    whitelist_rule_programs.list = vec![];

    let smart_wallet_seq = &mut ctx.accounts.smart_wallet_seq;
    smart_wallet_seq.seq = 0;
    Ok(())
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub signer: Signer<'info>,

    #[account(
        init,
        payer = signer,
        space = 8 + WhitelistRulePrograms::INIT_SPACE,
        seeds = [WhitelistRulePrograms::PREFIX_SEED],
        bump
    )]
    pub whitelist_rule_programs: Box<Account<'info, WhitelistRulePrograms>>,

    #[account(
        init,
        payer = signer,
        space = 8 + SmartWalletSeq::INIT_SPACE,
        seeds = [SmartWalletSeq::PREFIX_SEED],
        bump
    )]
    pub smart_wallet_seq: Box<Account<'info, SmartWalletSeq>>,

    pub system_program: Program<'info, System>,
}
