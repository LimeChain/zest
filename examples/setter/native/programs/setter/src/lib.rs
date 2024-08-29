use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    declare_id,
    entrypoint::ProgramResult,
    msg,
    program_error::ProgramError,
    pubkey::Pubkey,
};

pub mod state;
use solana_sdk::instruction::{AccountMeta, Instruction};
use state::*;

declare_id!("2mSfrzvHWwY1waEPtPNmdp3R1rESxmp6rx4ZhEUSVSJh");

#[cfg(not(feature = "no-entrypoint"))]
use solana_program::entrypoint;

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let (instruction_discriminant, instruction_data_inner) =
        instruction_data.split_at(1);
    match instruction_discriminant[0] {
        0 => {
            msg!("Instruction: Set");
            process_setter(accounts, instruction_data_inner)?;
        }
        _ => {
            msg!("Error: unknown instruction")
        }
    }
    Ok(())
}

pub fn process_setter(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    let account_info_iter = &mut accounts.iter();

    let setter_account = next_account_info(account_info_iter)?;
    assert!(
        setter_account.is_writable,
        "Counter account must be writable"
    );

    let mut setter =
        SetSetter::try_from_slice(&setter_account.try_borrow_mut_data()?)?;
    let text = std::str::from_utf8(instruction_data)
        .map_err(|_| ProgramError::Custom(14))?;
    setter.text = text.to_string();
    setter.serialize(&mut *setter_account.data.borrow_mut())?;

    msg!("Setter state set to {:?}", setter.text);
    Ok(())
}

pub fn text_pda(owner: &Pubkey) -> Pubkey {
    let (pda, _chat_bump) =
        Pubkey::find_program_address(&[owner.as_ref()], &ID);
    pda
}

pub fn set_instruction(
    sender: Pubkey,
    text_pda: Pubkey,
    text: String,
) -> Instruction {
    Instruction::new_with_borsh(
        ID,
        &SetSetter { text },
        vec![
            AccountMeta::new(text_pda, false),
            AccountMeta::new(sender, true),
            AccountMeta::new(solana_program::system_program::ID, false),
        ],
    )
}
