#![allow(clippy::result_large_err)]

use anchor_lang::prelude::*;

declare_id!("BmDHboaj1kBUoinJKKSRqKfMeRKJqQqEbUj1VgzeQe4A");

#[program]
pub mod counter_anchor {
    use super::*;

    pub fn increment(ctx: Context<CounterContext>) -> Result<()> {
        ctx.accounts.counter_pda.count =
            ctx.accounts.counter_pda.count.checked_add(1).unwrap();
        Ok(())
    }
}

#[derive(Accounts)]
#[instruction()]
pub struct CounterContext<'info> {
    #[account(
        init,
        seeds = [payer.key().as_ref()],
        bump,
        space = 8 + 8,
        payer = payer
    )]
    pub counter_pda: Account<'info, CounterState>,

    #[account(mut)]
    pub payer: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[account]
pub struct CounterState {
    pub count: u64,
}
