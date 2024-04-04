use anchor_lang::prelude::*;

#[error_code]
pub enum NoviError {
    #[msg("Overflow")]
    Overflow,
    #[msg("Underflow")]
    Underflow,

    #[msg("CreateIndex Instruction: You don't have the power to do that")]
    PrivilageEscalated,

    #[msg("Deposit Instruction: You surpassed the maximum Threshold")]
    MaxThreshold,
    #[msg("Deposit Instruction: You're short of the minimum Threshold")]
    MinThreshold,
    #[msg("Deposit Instruction: You're not using an allowed Mint")]
    InvalidMint,
    
    #[msg("InitializeSwap Instruction: You already swapped this token")]
    AlreadySwapped,
    #[msg("InitializeSwap Instruction: No CPI allowed")]
    CpiDisabled,
    #[msg("InitializeSwap Instruction: There is no Swap Instruction after this instruction")]
    InvalidSwapIx,
    #[msg("InitializeSwap Instruction: The Swap Instruction has more than 50bps of slippage")]
    InvalidSlippage,
    #[msg("InitializeSwap Instruction: The Swap Instruction has the wrong amount")]
    InvalidAmount,
    #[msg("InitializeSwap Instruction: The Swap Instruction has the wrong "From" Mint Address")]
    InvalidFromMint,
    #[msg("InitializeSwap Instruction: The Swap Instruction has the wrong "To" Mint Address")]
    InvalidToMint,
    #[msg("InitializeSwap Instruction: The Swap Instruction is missing")]
    MissingSwapIx,
    #[msg("InitializeSwap Instruction:  There is no Finalize Instruction after the Swap Instruction")]
    InvalidFinalizeIx,
    #[msg("InitializeSwap Instruction: The Finalize Instruction is passing in the wrong amount")]
    InvalidFinalizeAmount,
    #[msg("InitializeSwap Instruction: The Finalize Instruction is missing")]
    MissingFinalizeIx,
    #[msg("InitializeSwap Instruction: The Finalize Instruction has the wrong Owner")]
    InvalidFinalizeOwner,
    #[msg("InitializeSwap Instruction: The Finalize Instruction has the wrong Mint Address")]
    InvalidFinalizeMint,

    #[msg("Deposit Account >> Deposit: Amounts do not match")]
    AmountMismatch,

    #[msg("Index Account >> CheckAddress: The Mint passed in is Invalid")]
    InvalidMintAddress,
}