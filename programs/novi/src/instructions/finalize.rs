use anchor_lang::{
    prelude::*, 
    system_program::{create_account, CreateAccount},
}; 
use anchor_spl::{
    associated_token::AssociatedToken, token::{Mint, Token, TokenAccount, Transfer}
};

use crate::{
    errors::NoviError,
    state::{IndexAccount, IndexProfile},
};

#[derive(Accounts)]
pub struct Finalize<'info> {
    #[account(mut)]
    pub swapper: Signer<'info>,
    #[account(mut)]
    pub owner: SystemAccount<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"index", index.title.as_bytes()],
        bump = index.bump,
    )]
    pub index: Account<'info, IndexAccount>,
    #[account(
        seeds = [b"profile", index.key().as_ref(), owner.key().as_ref()],
        bump
    )]
    /// CHECK: This is fine because we're checking it in the instruction
    pub index_profile: AccountInfo<'info>,

    pub mint: Account<'info, Mint>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = index,
    )]
    pub index_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = swapper,
    )]
    pub swapper_token: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

impl<'info> Finalize<'info> {        
    pub fn finalize(&mut self, amount: u64, bumps: FinalizeBumps) -> Result<()> {

        // Deposit Swapped Funds to the Index Vault
        let index = self.index.clone();
        index.deposit(
            amount, 
            Transfer {
                from: self.swapper_token.to_account_info(),
                to: self.index_token.to_account_info(),
                authority: self.swapper.to_account_info(),
            },
            self.token_program.to_account_info(),
        )?;

        // Try and deserialize the Profile, if it fails, initialize it.
        let index_profile = self.index_profile.clone();
        let info = index_profile.to_account_info();
        let data = info.try_borrow_mut_data()?;
        match IndexProfile::try_deserialize(&mut &data[..]) {
            Ok(_) => {
                let mut profile = IndexProfile::try_from_slice(&index_profile.data.borrow()).unwrap();
                profile.mint_amount[index.check_address(self.mint.key())?] = profile.mint_amount[index.check_address(self.mint.key())?].checked_add(amount).ok_or(NoviError::Overflow)?;
            },
            Err(_) => {
                create_account(
                    CpiContext::new(
                        self.system_program.to_account_info(), 
                        CreateAccount {
                            from: self.payer.to_account_info(),
                            to: index_profile.to_account_info(),                    
                        }), 
                    Rent::get()?.minimum_balance(IndexProfile::INIT_SPACE), 
                    IndexProfile::INIT_SPACE as u64, 
                    &crate::ID,
                )?;
                
                let mut profile = IndexProfile::try_from_slice(&index_profile.data.borrow()).unwrap();
                profile.owner = self.owner.key();
                let mut mint_amount: Vec<u64> = Vec::new();
                for _ in 0..self.index.mint_list.len() {
                    mint_amount.push(0);
                };
                mint_amount[index.check_address(self.mint.key())?] = amount;
                profile.mint_amount = mint_amount;
                profile.bump = bumps.index_profile;
                profile.serialize(&mut *index_profile.data.borrow_mut())?;
            }
        }

        Ok(())
    }
}