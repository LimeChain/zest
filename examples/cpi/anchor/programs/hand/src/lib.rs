#![allow(clippy::result_large_err)]

use anchor_lang::{prelude::*, InstructionData};
use lever::cpi::accounts::SetPowerStatus;
use lever::program::Lever;
use lever::{self, PowerStatus};
use solana_program::instruction::Instruction;

declare_id!("EJfTLXDCJTVwBgGpz9X2Me4CWHbvg8F8zsM7fiVJLLeR");

#[derive(Accounts)]
// NOTE: (seemingly) has no effect
// #[instruction(name: String)]
pub struct PullLever<'info> {
    #[account(mut)]
    pub power: Account<'info, PowerStatus>,
    pub lever_program: Program<'info, Lever>,
}

#[program]
pub mod hand {
    use super::*;

    pub fn pull_lever(ctx: Context<PullLever>, name: String) -> Result<()> {
        // Hitting the switch_power method on the lever program
        //
        lever::cpi::switch_power(
            CpiContext::new(
                ctx.accounts.lever_program.to_account_info(),
                // Using the accounts context struct from the lever program
                //
                SetPowerStatus {
                    power: ctx.accounts.power.to_account_info(),
                },
            ),
            name,
        )
    }
}

pub fn pull_instruction(power: Pubkey, who: String) -> Instruction {
    let instruction = instruction::PullLever { name: who };

    Instruction::new_with_bytes(
        ID,
        &instruction.data(),
        vec![
            AccountMeta::new(power, false),
            AccountMeta::new_readonly(lever::ID, false),
        ],
    )
}
