use anchor_lang::prelude::*;

pub mod instructions;
pub mod programs;
pub mod errors;
pub mod macros;
pub mod constants;
pub mod state;

use instructions::*;

declare_id!("FuXing9rWvKB8zPtnUCeJGMQT4CUJx6BVVwE8XnBLPtw");

#[program]
pub mod novi {
    use super::*;

    pub fn finalize(ctx: Context<Finalize>, amount: u64) -> Result<()> {
        ctx.accounts.finalize(amount, ctx.bumps)
    }
}
