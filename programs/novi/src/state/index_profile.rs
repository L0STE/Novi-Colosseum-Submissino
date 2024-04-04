use anchor_lang::prelude::*;

#[account]
pub struct IndexProfile {
    pub owner: Pubkey,
    pub mint_amount: Vec<u64>,
    pub bump: u8,
}

impl Space for IndexProfile {
    const INIT_SPACE: usize = 8 + 32 + 4 + 1;
}