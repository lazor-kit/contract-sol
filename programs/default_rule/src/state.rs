use anchor_lang::prelude::*;

#[account]
#[derive(Debug, InitSpace)]
pub struct Rule {
    pub smart_wallet: Pubkey,
    pub passkey_pubkey: [u8; 33],
    pub is_initialized: bool,
}

#[account]
#[derive(Debug, InitSpace)]
pub struct Config {
    pub authority: Pubkey,
}
