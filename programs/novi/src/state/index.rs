use anchor_lang::prelude::*;
use anchor_spl::token::{transfer, Transfer};
use std::result::Result as StdResult;
use crate::errors::NoviError;

#[account]
pub struct IndexAccount {
    pub title: String,
    pub mint_list: Vec<Pubkey>,
    pub bump: u8,
}

impl Space for IndexAccount {
    const INIT_SPACE: usize = 8 + 4 + 4 + 1;
}

impl IndexAccount {
    pub fn initialize(&mut self, title: String, mint_list: Vec<Pubkey>, bump: u8) {
        self.title = title;
        self.mint_list = mint_list;
        self.bump = bump;
    }

    pub fn check_address(&self, mint: Pubkey) -> StdResult<usize, NoviError> {
        match self.mint_list.iter().position(|&m| m == mint) {
            Some(index) => Ok(index),
            None => Err(NoviError::InvalidMintAddress), 
        }
    }

    pub fn deposit<'info>(
        &self,
        amount: u64,
        accounts: Transfer<'info>,
        program: AccountInfo<'info>,
    ) -> Result<()> {        
        transfer(CpiContext::new(program, accounts), amount)
    }

    pub fn withdraw<'info>(
        &mut self,
        amount: u64,
        accounts: Transfer<'info>,
        program: AccountInfo<'info>,
        signer_seeds: &[&[&[u8]]]
    ) -> Result<()> {
        transfer(CpiContext::new_with_signer(program, accounts, signer_seeds), amount)
    }
}