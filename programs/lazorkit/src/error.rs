use anchor_lang::error_code;

#[error_code]
pub enum LazorKitError {
    #[msg("This hook is not whitelisted")]
    HookNotWhitelisted,

    #[msg("Invalid verify instruction length")]
    InvalidLengthForVerification,

    VerifyHeaderMismatchError,

    VerifyDataMismatchError,

    InvalidPasskey,

    InvalidHook,

    InvalidBump,

    InvalidRuleProgram,
}
