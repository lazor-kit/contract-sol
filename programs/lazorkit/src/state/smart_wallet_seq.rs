use anchor_lang::prelude::*;

#[account]
#[derive(Debug, InitSpace)]
pub struct SmartWalletSeq {
    pub seq: u64,
}

impl SmartWalletSeq {
    pub const PREFIX_SEED: &'static [u8] = b"smart_wallet_seq";
}
