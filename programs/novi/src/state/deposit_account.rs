use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Transfer};
use solana_program::system_program;

use crate::errors::NoviError;

#[account]
pub struct DepositAccount {
    pub owner: Pubkey,
    pub amount: u64,
    pub mint_list: Vec<bool>,
    pub seed: u64,
    pub bump: u8,
}

impl Space for DepositAccount {
    const INIT_SPACE: usize = 8 + 32 + 8 + 4 + 8 + 1;
}

impl DepositAccount {
    pub fn initialize(&mut self, owner: Pubkey, amount: u64, mint: Vec<bool>, seed: u64, bump: u8) {
        self.owner = owner;
        self.amount = amount;
        self.mint_list = mint;
        self.seed = seed;
        self.bump = bump;
    }

    pub fn deposit<'info>(
        &self,
        amount: u64,
        accounts: Transfer<'info>,
        program: AccountInfo<'info>,
    ) -> Result<()> {
        require_eq!(amount, self.amount, NoviError::AmountMismatch);
        
        transfer(CpiContext::new(program, accounts), amount)
    }

    pub fn withdraw<'info>(
        &mut self,
        amount: u64,
        accounts: Transfer<'info>,
        program: AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]]
    ) -> Result<()> {
        require_eq!(amount, self.amount.checked_div(self.mint_list.len() as u64).ok_or(NoviError::Overflow)?, NoviError::AmountMismatch);        
        transfer(CpiContext::new_with_signer(program, accounts, signer_seeds), amount)
    }

    pub fn close<'info>(deposit: AccountInfo<'info>, payer: AccountInfo<'info>) -> Result<()> {
        let dest_starting_lamports = payer.lamports();
        **payer.lamports.borrow_mut() = dest_starting_lamports.checked_add(deposit.lamports()).unwrap();
        **deposit.lamports.borrow_mut() = 0;
    
        deposit.assign(&system_program::ID);
        deposit.realloc(0, false).map_err(Into::into)
    }
}