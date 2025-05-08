use anchor_lang::prelude::*;

#[account]
#[derive(Debug, InitSpace)]
pub struct WhitelistRulePrograms {
    #[max_len(32)]
    pub list: Vec<Pubkey>,
}

impl WhitelistRulePrograms {
    pub const PREFIX_SEED: &'static [u8] = b"whitelist_rule_programs";
}
