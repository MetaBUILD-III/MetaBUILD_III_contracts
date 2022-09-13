use near_sdk::{ext_contract, json_types::U128, AccountId};

pub type WBalance = U128;
pub const FEE_DIVISOR: u32 = 10_000;
pub const MARKET_PLATFORM_ACCOUNT: &str = "omomo.nearlend.testnet";
