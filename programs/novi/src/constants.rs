pub const MIN_SOL_THRESHOLD: u64 = 0;
pub const MAX_SOL_THRESHOLD: u64 = 1_000_000_000_000_000;
pub const MIN_USD_THRESHOLD: u64 = 0;
pub const MAX_USD_THRESHOLD: u64 = 1_000_000_000_000_000;

use anchor_lang::declare_id;

pub mod admin {
    use super::*;
    declare_id!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"); // ToDo
}

pub mod usdc {
    use super::*;
    declare_id!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
}

pub mod usdt {
    use super::*;
    declare_id!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
}

pub mod wsol {
    use super::*;
    declare_id!("So11111111111111111111111111111111111111112");
}