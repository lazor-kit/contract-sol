use anchor_lang::prelude::*;

mod constants;
mod error;
mod instructions;
mod state;
mod utils;

use instructions::*;

declare_id!("HJoSAFHenQfaYuMgYZ8ZfhsRsuSZ8WYDSVm788DqvVEw");

#[program]
pub mod lazorkit {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        instructions::initialize(ctx)
    }

    pub fn upsert_whitelist_rule_programs(
        ctx: Context<UpsertWhitelistRulePrograms>,
        hook: Pubkey,
    ) -> Result<()> {
        instructions::upsert_whitelist_rule_programs(ctx, hook)
    }

    pub fn create_smart_wallet(
        ctx: Context<CreateSmartWallet>,
        args: CreateSmartWalletArgs,
    ) -> Result<()> {
        instructions::create_smart_wallet(ctx, args)
    }

    pub fn execute_instruction(
        ctx: Context<ExecuteInstruction>,
        args: ExecuteInstructionArgs,
    ) -> Result<()> {
        instructions::execute_instruction(ctx, args)
    }
}
