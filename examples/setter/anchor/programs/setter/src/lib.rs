#![allow(clippy::result_large_err)]

use anchor_lang::{prelude::*, InstructionData};
use solana_program::instruction::Instruction;

declare_id!("2mSfrzvHWwY1waEPtPNmdp3R1rESxmp6rx4ZhEUSVSJh");

#[account]
pub struct SetterState {
    pub text: String,
}

// https://book.anchor-lang.com/anchor_references/space.html
const MAX_STRING_BYTES: usize = 255;

#[derive(Accounts)]
#[instruction(text: String)]
pub struct SetContext<'info> {
    #[account(
        init_if_needed,
        seeds = [initializer.key().as_ref()],
        bump,
        payer = initializer,
        space = MAX_STRING_BYTES
    )]
    pub text_pda: Account<'info, SetterState>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[program]
pub mod coverage {
    use super::*;

    #[allow(unused_variables)] // `message_seed` used in `init` of `SendDirectMessage`
    pub fn set_context(ctx: Context<SetContext>, text: String) -> Result<()> {
        let text_pda = &mut ctx.accounts.text_pda;

        if text.contains("Error") {
            return Err(anchor_lang::solana_program::program_error::ProgramError::Custom(123).into());
        }

        #[allow(clippy::if_same_then_else)]
        if text.is_empty() {
            text_pda.text = "Default message".to_string();
        } else if false {
            text_pda.text = text;
        } else if true {
            text_pda.text = text;
        } else {
            text_pda.text = text;
        }

        Ok(())
    }
}

pub fn text_pda(owner: &Pubkey) -> Pubkey {
    let (pda, _chat_bump) = Pubkey::find_program_address(&[owner.as_ref()], &ID);
    pda
}

pub fn set_instruction(sender: Pubkey, text_pda: Pubkey, text: String) -> Instruction {
    let instruction = instruction::SetContext { text };

    Instruction::new_with_bytes(
        ID,
        &instruction.data(),
        vec![
            AccountMeta::new(text_pda, false),
            AccountMeta::new(sender, true),
            AccountMeta::new(solana_program::system_program::ID, false),
        ],
    )
}
