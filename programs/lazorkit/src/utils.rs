use crate::constants::SECP256R1_ID;
use crate::{error::LazorKitError, ID};
use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::Instruction,
    program::{invoke, invoke_signed},
};

#[derive(AnchorSerialize, AnchorDeserialize, Clone)]
pub struct PdaSigner {
    pub seeds: Vec<u8>,
    pub bump: u8,
}

pub fn execute_cpi_instruction(
    accounts: &[AccountInfo],
    instruction_data: Vec<u8>,
    cpi_program: &AccountInfo,
    pda_signer: Option<PdaSigner>,
) -> Result<()> {
    let account_metas = accounts
        .iter()
        .map(|account| AccountMeta {
            is_signer: pda_signer.as_ref().map_or(account.is_signer, |pda| {
                account.key == &Pubkey::find_program_address(&[&pda.seeds], &ID).0
                    || account.is_signer
            }),
            is_writable: account.is_writable,
            pubkey: *account.key,
        })
        .collect::<Vec<_>>();

    let instruction = Instruction {
        program_id: cpi_program.key(),
        accounts: account_metas,
        data: instruction_data,
    };

    if let Some(pda) = pda_signer {
        let seeds = &[&pda.seeds[..], &[pda.bump]];
        invoke_signed(&instruction, accounts, &[seeds])?;
    } else {
        invoke(&instruction, accounts)?;
    }

    Ok(())
}

pub fn verify_secp256r1_ix(
    instruction: &Instruction,
    public_key: [u8; 33],
    message: Vec<u8>,
    signature: Vec<u8>,
) -> Result<()> {
    if instruction.program_id != SECP256R1_ID
        || instruction.accounts.len() != 0
        || instruction.data.len() != (2 + 14 + 33 + 64 + message.len())
    {
        return Err(LazorKitError::InvalidLengthForVerification.into());
    }

    check_secp256r1_data(&instruction.data, public_key, message, signature)?;
    Ok(())
}

fn check_secp256r1_data(
    data: &[u8],
    public_key: [u8; 33],
    message: Vec<u8>,
    signature: Vec<u8>,
) -> Result<()> {
    // Parse header components
    let num_signatures = &[data[0]];
    let signature_offset = &data[2..=3];
    let signature_instruction_index = &data[4..=5];
    let public_key_offset = &data[6..=7];
    let public_key_instruction_index = &data[8..=9];
    let message_data_offset = &data[10..=11];
    let message_data_size = &data[12..=13];
    let message_instruction_index = &data[14..=15];

    // Get actual data
    let parsed_public_key = &data[16..16 + 33];
    let parsed_signature = &data[49..49 + 64];
    let parsed_message = &data[113..];

    // Calculate expected values
    const SIGNATURE_OFFSETS_SERIALIZED_SIZE: u16 = 14;
    const DATA_START: u16 = 2 + SIGNATURE_OFFSETS_SERIALIZED_SIZE;
    let message_length: u16 = message.len() as u16;
    let public_key_length: u16 = public_key.len() as u16;
    let signature_length: u16 = signature.len() as u16;

    let expected_public_key_offset: u16 = DATA_START;
    let expected_signature_offset: u16 = DATA_START + public_key_length;
    let expected_message_data_offset: u16 = expected_signature_offset + signature_length;

    // Verify header
    if num_signatures != &[1]
        || signature_offset != &expected_signature_offset.to_le_bytes()
        || signature_instruction_index != &0xFFFFu16.to_le_bytes()
        || public_key_offset != &expected_public_key_offset.to_le_bytes()
        || public_key_instruction_index != &0xFFFFu16.to_le_bytes()
        || message_data_offset != &expected_message_data_offset.to_le_bytes()
        || message_data_size != &message_length.to_le_bytes()
        || message_instruction_index != &0xFFFFu16.to_le_bytes()
    {
        return Err(LazorKitError::VerifyHeaderMismatchError.into());
    }

    if &parsed_public_key[..] != &public_key[..]
        || &parsed_signature[..] != &signature[..]
        || &parsed_message[..] != &message[..]
    {
        return Err(LazorKitError::VerifyDataMismatchError.into());
    }
    Ok(())
}
