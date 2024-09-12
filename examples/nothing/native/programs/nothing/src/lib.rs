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
use state::*;

declare_id!("BagZvfJTVtWiJYcj82WxvC6NHV9njzDM1M2gF6HrC9YL");

#[cfg(not(feature = "no-entrypoint"))]
use solana_program::entrypoint;

#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    process_nothing()?;
    Ok(())
}

pub fn process_nothing(
    // _accounts: &[AccountInfo],
    // _instruction_data: &[u8],
) -> Result<(), ProgramError> {
    msg!("nothing did nothing");
    Ok(())
}
