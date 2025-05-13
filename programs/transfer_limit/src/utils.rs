use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::get_associated_token_address_with_program_id, token_interface::TokenAccount,
};

use crate::errors::TransferLimitError;

/// Helper function to get token account and balance
pub fn get_token_account_and_balance<'a: 'info, 'info>(
    smart_wallet: &Pubkey,
    token: &Pubkey,
    remaining_accounts: &mut std::slice::Iter<'a, AccountInfo<'info>>,
) -> Result<(&'a AccountInfo<'info>, u64)> {
    let token_program = next_account_info(remaining_accounts)?;
    let token_account = next_account_info(remaining_accounts)?;

    let expected_token_account =
        get_associated_token_address_with_program_id(smart_wallet, token, &token_program.key());

    require!(
        token_account.key() == expected_token_account,
        TransferLimitError::InvalidTokenAccount
    );

    let vault_token_account = InterfaceAccount::<TokenAccount>::try_from(token_account)?;
    Ok((token_account, vault_token_account.amount))
}

/// Helper function to close an account
pub fn close_account(source: &AccountInfo, destination: &AccountInfo) {
    let dest_starting_lamports = destination.lamports();
    let source_lamports = source.lamports();

    **destination.lamports.borrow_mut() =
        dest_starting_lamports.checked_add(source_lamports).unwrap();
    **source.lamports.borrow_mut() = 0;

    let mut source_data = source.data.borrow_mut();
    source_data.fill(0);
}
