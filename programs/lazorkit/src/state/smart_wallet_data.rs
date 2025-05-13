use anchor_lang::prelude::*;

/// Data account for a smart wallet
#[account]
#[derive(Default, InitSpace)]
pub struct SmartWalletData {
    /// Unique identifier for this smart wallet
    pub id: u64,
    /// Optional rule program that governs this wallet's operations
    pub rule_program: Option<Pubkey>,
    /// Bump seed for PDA derivation
    pub bump: u8,
}

impl SmartWalletData {
    pub const PREFIX_SEED: &'static [u8] = b"smart_wallet_data";
}
