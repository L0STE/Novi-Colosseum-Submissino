use anchor_lang::prelude::*;
use anchor_spl::{
    token::{Mint, Token, TokenAccount, Transfer},
    associated_token::AssociatedToken,
};
use crate::{
    state::{DepositAccount, IndexAccount},
    constants::{MIN_SOL_THRESHOLD, MIN_USD_THRESHOLD, MAX_SOL_THRESHOLD, MAX_USD_THRESHOLD, usdc, usdt, wsol},
    errors::NoviError,
};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        init,
        payer = payer,
        seeds = [b"deposit", seed.to_le_bytes().as_ref(), user.key().as_ref()],
        bump,    
        space = DepositAccount::INIT_SPACE + index.mint_list.len() * 1,
    )]
    pub deposit: Account<'info, DepositAccount>,
    #[account(
        mut,
        seeds = [b"index", index.title.as_bytes()],
        bump = index.bump,
    )]
    pub index: Account<'info, IndexAccount>,

    pub mint: Account<'info, Mint>,
    #[account(
        init,
        payer = payer,
        associated_token::mint = mint,
        associated_token::authority = deposit,
    )]
    pub deposit_token: Account<'info, TokenAccount>,
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = user,
    )]
    pub user_token: Account<'info, TokenAccount>,

    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

impl<'info> Deposit<'info> {        
    pub fn deposit(&mut self, seed: u64, amount: u64, bumps: DepositBumps) -> Result<()> {

        // We check that the Mint is correct and that the Amoun is within the threshold
        if self.mint.key() == wsol::id() {
            require!(amount >= MIN_SOL_THRESHOLD, NoviError::MinThreshold);
            require!(amount <= MAX_SOL_THRESHOLD, NoviError::MaxThreshold);
        } else if self.mint.key() == usdc::id() {
            require!(amount >= MIN_USD_THRESHOLD, NoviError::MinThreshold);
            require!(amount <= MAX_USD_THRESHOLD, NoviError::MaxThreshold);
        } else if self.mint.key() == usdt::id() {
            require!(amount >= MIN_USD_THRESHOLD, NoviError::MinThreshold);
            require!(amount <= MAX_USD_THRESHOLD, NoviError::MaxThreshold); 
        } else {
            return Err(NoviError::InvalidMint.into());
        }

        let mut mint_list: Vec<bool> = Vec::new();
        for _ in 0..self.index.mint_list.len() {
            mint_list.push(false);
        }
        
        // We initialize the DepositAccount and Deposit the funds
        let mut deposit_account = self.deposit.clone();
        deposit_account.initialize(self.user.key(), amount, mint_list, seed, bumps.deposit);
        deposit_account.deposit(
            amount.checked_mul(self.mint.decimals as u64).ok_or(NoviError::Overflow)?, 
            Transfer {
                from: self.user_token.to_account_info(),
                to: self.deposit_token.to_account_info(),
                authority: self.user.to_account_info(),
            }, 
            self.token_program.to_account_info()
        )?;

        Ok(())
    }
}