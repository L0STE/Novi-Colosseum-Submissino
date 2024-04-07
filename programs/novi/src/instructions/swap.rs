use anchor_lang::{prelude::*, Discriminator}; 
use anchor_spl::{
    associated_token::AssociatedToken, 
    token::{close_account, CloseAccount, Mint, Token, TokenAccount, Transfer}
};
use solana_program::{
    instruction::{get_stack_height, TRANSACTION_LEVEL_STACK_HEIGHT},
    sysvar::{self, instructions}
};

use crate::{
    errors::NoviError, programs::jupiter::{
        self, SharedAccountsRoute
    }, require_discriminator_eq, require_instruction_eq, state::{DepositAccount, IndexAccount}
};

#[derive(Accounts)]
pub struct InitializeSwap<'info> {
    #[account(mut)]
    pub swapper: Signer<'info>,
    #[account(mut)]
    pub payer: Signer<'info>,

    #[account(
        mut,
        seeds = [b"deposit", deposit.seed.to_le_bytes().as_ref(), deposit.owner.as_ref()],
        bump = deposit.bump,    
    )]
    pub deposit: Account<'info, DepositAccount>,
    #[account(
        mut,
        seeds = [b"index", index.title.as_bytes()],
        bump = index.bump,
    )]
    pub index: Account<'info, IndexAccount>,

    pub usdc: Account<'info, Mint>,
    #[account(
        mut,
        associated_token::mint = usdc,
        associated_token::authority = deposit,
    )]
    pub deposit_token: Account<'info, TokenAccount>,
    #[account(
        init_if_needed,
        payer = payer,
        associated_token::mint = usdc,
        associated_token::authority = swapper,
    )]
    pub swapper_token: Account<'info, TokenAccount>,
    pub mint: Account<'info, Mint>,

    #[account(address = sysvar::instructions::ID)]
    /// CHECK: InstructionsSysvar account
    pub instructions_sysvar_program: UncheckedAccount<'info>,
    pub associated_token_program: Program<'info, AssociatedToken>,
    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>
}

impl<'info> InitializeSwap<'info> {        
    pub fn initialize_swap(&mut self, amount: u64) -> Result<()> {
        let index = self.index.clone();
        let mut deposit = self.deposit.clone();

        // Check if the Mint is in the IndexAccount mint_list and log at what position is.
        let mint_index = index.check_address(self.mint.key())?;
        require!(deposit.mint_list[mint_index] == false, NoviError::AlreadySwapped);        

        // Transfer the tokens from the deposit to the swapper
        let index_bump_slice: &[u8] = &[index.bump];

        let signer_seeds = &[&[b"index".as_ref(), index.title.as_bytes(), index_bump_slice][..]];
        deposit.withdraw(
            amount, 
            Transfer {
                from: self.deposit_token.to_account_info(),
                to: self.swapper_token.to_account_info(),
                authority: self.deposit.to_account_info(),
            },
            self.token_program.to_account_info(),
            signer_seeds,
        )?;

        // Close the deposit_token and deposit if there is no USDC in the vault
        let deposit_seed_bytes = deposit.seed.to_le_bytes();
        let deposit_bump_slice: &[u8] = &[deposit.bump];
        
        if self.deposit_token.amount == 0 {
            let signer_seeds = &[&[b"deposit".as_ref(), deposit_seed_bytes.as_ref(), deposit.owner.as_ref(), deposit_bump_slice][..]];
            close_account(
                CpiContext::new_with_signer(
                    self.token_program.to_account_info(),
                    CloseAccount {
                        account: self.deposit_token.to_account_info(),
                        destination: self.payer.to_account_info(),
                        authority: self.deposit.to_account_info(),
                    },
                    signer_seeds
                )
            )?;
            deposit.close(self.payer.to_account_info())?;
        }

        /* 
        
            Instruction Introspection

            This is the primary means by which we secure our program,
            enforce atomicity while making a great UX for our users.

        */

        let ixs = self.instructions_sysvar_program.to_account_info();
        let current_index: usize = instructions::load_current_index_checked(&ixs)?.into();

        /*

            Disable CPIs
            
            Although we have taken numerous measures to secure this program,
            we can kill CPI to close off even more attack vectors as our 
            current use case doesn't need it.

        */

        require_eq!(get_stack_height(), TRANSACTION_LEVEL_STACK_HEIGHT, NoviError::CpiDisabled);

        /* 
        
            Match Jupiter Swap Instruction
            
            Ensure that the next instruction after this one is a swap in the
            Jupiter program. Checks include:

            - Program ID and IX discriminator
            - Token account matching
            - Mint account matching
            - Deposit amount matching
            - Minimum SOL amount matching
            - Max slippage protection

            By matching token accounts against our account struct which already 
            enforces mint constraints, we should be able to deduce the mint
            accounts in the instruction also match. Alas, we check them anyway
            just to be extra safe.

            Basically, the only way this rugs is if Jupiter gets hacked.

        */

        let quoted_out_amount: u64;
        
        if let Ok(ix) = instructions::load_instruction_at_checked(current_index + 1, &ixs) {
            // Check Swap Instruction
            require_instruction_eq!(ix, jupiter::ID, SharedAccountsRoute::DISCRIMINATOR, NoviError::InvalidSwapIx);
            let shared_account_route_ix = SharedAccountsRoute::try_from_slice(&ix.data[8..])?;
            require_eq!(shared_account_route_ix.slippage_bps, 50, NoviError::InvalidSlippage);
            require_eq!(shared_account_route_ix.in_amount, amount, NoviError::InvalidAmount);
            quoted_out_amount = shared_account_route_ix.quoted_out_amount.checked_mul(self.mint.decimals.into()).ok_or(NoviError::Overflow)?;

            // Check if the "From" and "To" mint address
            require_keys_eq!(ix.accounts.get(7).ok_or(NoviError::InvalidFromMint)?.pubkey, self.usdc.key(), NoviError::InvalidFromMint);
            require_keys_eq!(ix.accounts.get(8).ok_or(NoviError::InvalidToMint)?.pubkey, self.mint.key(), NoviError::InvalidToMint);
        } else {
            return Err(NoviError::MissingSwapIx.into());
        }

        /* 
        
            Match Finalize Instruction
            
            We also ensure that the instruction after swapping on Jupiter is the
            finalize instruction. Checks include:

            - Program ID and IX discriminator
            - Owner Matching
            - Quoted_out_amount Matching
            - Mint Matching

        */

        if let Ok(ix) = instructions::load_instruction_at_checked(current_index + 2, &ixs) {
            // Check Finalize Instruction
            require_instruction_eq!(ix, crate::ID, crate::instruction::Finalize::DISCRIMINATOR, NoviError::InvalidFinalizeIx);

            // Data Check
            let finalize_ix = crate::instruction::Finalize::try_from_slice(&ix.data[8..])?;
            require_eq!(finalize_ix.amount, quoted_out_amount, NoviError::InvalidFinalizeAmount);

            // Account Check
            require_keys_eq!(ix.accounts.get(1).ok_or(NoviError::InvalidFinalizeOwner)?.pubkey, self.deposit.owner, NoviError::InvalidFinalizeOwner);
            require_keys_eq!(ix.accounts.get(5).ok_or(NoviError::InvalidFinalizeMint)?.pubkey, self.mint.key(), NoviError::InvalidFinalizeMint);
        } else {
            return Err(NoviError::MissingFinalizeIx.into());
        }

        Ok(())
    }
}