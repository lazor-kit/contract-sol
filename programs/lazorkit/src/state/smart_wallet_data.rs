use anchor_lang::prelude::*;

#[account]
#[derive(Debug, InitSpace)]
pub struct SmartWalletData {
    pub rule_program: Option<Pubkey>,
    pub id: u64,
    pub bump: u8,
}

impl SmartWalletData {
    pub const PREFIX_SEED: &'static [u8] = b"smart_wallet_data";
}
