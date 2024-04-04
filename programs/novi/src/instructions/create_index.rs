use anchor_lang::prelude::*;

use crate::{
    state::IndexAccount, 
    constants::admin, 
    errors::NoviError
};

#[derive(Accounts)]
#[instruction(title: String, mint_list: Vec<Pubkey>)]
pub struct CreateIndex<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [b"index", title.as_bytes()],
        bump,
        space = IndexAccount::INIT_SPACE + title.len() + mint_list.len() * 32,
    )]
    pub index: Account<'info, IndexAccount>,
    
    pub system_program: Program<'info, System>
}

impl<'info> CreateIndex<'info> {        
    pub fn create(&mut self, title: String, mint_list: Vec<Pubkey>, bumps: CreateIndexBumps) -> Result<()> {
        require!(self.admin.key() == admin::id(), NoviError::PrivilageEscalated);
        
        let index = &mut self.index;
        index.initialize(title, mint_list, bumps.index);

        Ok(())
    }
}