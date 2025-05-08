use anchor_lang::prelude::*;

#[account]
#[derive(Debug, InitSpace)]
pub struct SmartWalletAuthenticator {
    pub passkey_pubkey: [u8; 33],
    pub smart_wallet: Pubkey,
    pub nonce: u64,
    pub bump: u8,
}

impl SmartWalletAuthenticator {
    pub const PREFIX_SEED: &'static [u8] = b"smart_wallet_authenticator";
}
