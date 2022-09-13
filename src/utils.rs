use crate::*;

use near_sdk::{
    ext_contract, json_types::U128, AccountId,
};

pub type WBalance = U128;

pub const FEE_DIVISOR: u32 = 10_000;

pub type Digits = u32;

pub const TGAS: Gas = near_sdk::Gas::ONE_TERA;
pub const MARKET_PLATFORM_ACCOUNT: &str = "omomo.nearlend.testnet";

#[ext_contract(ext_token)]
pub(crate) trait NEP141Token {
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: WBalance,
        memo: Option<String>,
        msg: String,
    );

    fn ft_transfer(&mut self, receiver_id: AccountId, amount: WBalance, memo: Option<String>);
}

impl Contract {
    pub fn is_valid_market_call(&self) -> bool {
        self.markets.contains(&env::predecessor_account_id())
    }

    pub fn terra_gas(&self, gas: u64) -> Gas {
        TGAS * gas
    }
}