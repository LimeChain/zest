#![allow(clippy::result_large_err)]

use anchor_lang::{prelude::*, InstructionData};
use solana_program::instruction::Instruction;

declare_id!("2mSfrzvHWwY1waEPtPNmdp3R1rESxmp6rx4ZhEUSVSJh");

#[derive(Accounts)]
#[instruction()]
pub struct NothingContext<'info> {
    #[account(mut)]
    pub initializer: Signer<'info>,
}

#[program]
pub mod coverage {
    use super::*;

    pub fn nothing_context(
        _ctx: Context<NothingContext>,
    ) -> Result<()> {
        Ok(())
    }
}

pub fn text_pda(owner: &Pubkey) -> Pubkey {
    let (pda, _chat_bump) =
        Pubkey::find_program_address(&[owner.as_ref()], &ID);
    pda
}

pub fn nothing_instruction(sender: Pubkey) -> Instruction {
    let instruction = instruction::NothingContext { };

    Instruction::new_with_bytes(
        ID,
        &instruction.data(),
        vec![AccountMeta::new(sender, true)],
    )
}
