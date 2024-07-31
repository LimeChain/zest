#![allow(clippy::result_large_err)]

use anchor_lang::{prelude::*, system_program, InstructionData};
use solana_program::instruction::Instruction;

declare_id!("CABVoybzrbAJSv7QhQd6GXNGKxDMRjw9niqFzizhk6uk");

#[derive(Accounts)]
pub struct InitializeLever<'info> {
    #[account(
        init_if_needed,
        seeds = [initializer.key().as_ref()],
        bump,
        payer = initializer,
        space = 8 + 8
    )]
    pub power: Account<'info, PowerStatus>,
    #[account(mut)]
    pub initializer: Signer<'info>,
    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct SetPowerStatus<'info> {
    #[account(mut)]
    pub power: Account<'info, PowerStatus>,
}

#[account]
pub struct PowerStatus {
    pub is_on: bool,
}

#[program]
pub mod lever {
    use super::*;

    pub fn initialize(_ctx: Context<InitializeLever>) -> Result<()> {
        msg!("Lever initialized!");

        Ok(())
    }

    pub fn switch_power(ctx: Context<SetPowerStatus>, name: String) -> Result<()> {
        let power = &mut ctx.accounts.power;
        power.is_on = !power.is_on;

        msg!("{} is pulling the power switch!", &name);

        match power.is_on {
            true => msg!("The power is now on."),
            false => msg!("The power is now off!"),
        };

        Ok(())
    }
}

pub fn power_pda(owner: &Pubkey) -> Pubkey {
    let (pda, _chat_bump) = Pubkey::find_program_address(&[owner.as_ref()], &ID);
    pda
}

pub fn init_instruction(sender: Pubkey, power: Pubkey) -> Instruction {
    let instruction = instruction::Initialize {};

    Instruction::new_with_bytes(
        ID,
        &instruction.data(),
        vec![
            AccountMeta::new(power, false),
            AccountMeta::new(sender, true),
            AccountMeta::new(system_program::ID, false),
        ],
    )
}

pub fn switch_instruction(power: Pubkey, who: String) -> Instruction {
    let instruction = instruction::SwitchPower { name: who };

    Instruction::new_with_bytes(
        ID,
        &instruction.data(),
        vec![
            AccountMeta::new(power, false),
        ],
    )
}
