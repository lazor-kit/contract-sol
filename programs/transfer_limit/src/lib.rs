use anchor_lang::prelude::*;

mod errors;
mod instructions;
mod state;
mod utils;

use instructions::*;

declare_id!("HjgdxTNPqpL59KLRVDwQ28cqam2SxBirnNN5SFAFGHZ8");

#[program]
pub mod transfer_limit {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>, lazorkit_author: Pubkey) -> Result<()> {
        instructions::initialize(ctx, lazorkit_author)
    }

    pub fn init_rule(ctx: Context<InitRule>, init_rule_args: InitRuleArgs) -> Result<()> {
        instructions::init_rule(ctx, init_rule_args)
    }

    pub fn add_member(ctx: Context<AddMember>, new_passkey_pubkey: [u8; 33]) -> Result<()> {
        instructions::add_member(ctx, new_passkey_pubkey)
    }

    pub fn execute_instruction<'c: 'info, 'info>(
        ctx: Context<'_, '_, 'c, 'info, ExecuteInstruction<'info>>,
        args: ExecuteInstructionArgs,
    ) -> Result<()> {
        instructions::execute_instruction(ctx, args)
    }
}
