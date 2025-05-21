use anchor_lang::prelude::*;

declare_id!("B98ooLRYBP6m6Zsrd3Hnzn4UAejfVZwyDgMFaBNzVR2W");

mod error;
mod instructions;
mod state;

use instructions::*;

#[program]
pub mod default_rule {

    use super::*;

    pub fn upsert_rule(ctx: Context<CreateRule>, auth_passkey_pubkey: [u8; 33]) -> Result<()> {
        instructions::upsert_rule(ctx, auth_passkey_pubkey)
    }
}
