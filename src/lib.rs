mod big_decimal;
mod cancel_order;
mod common;
mod deposit;
mod ft;
mod market;
mod metadata;
mod price;
mod view;

use crate::metadata::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, require, AccountId, Balance};
use std::collections::HashMap;

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// market ➝ MarketData
    market_infos: LookupMap<AccountId, MarketData>,

    /// Protocol fee
    protocol_fee: u128,

    /// token ➝ Price
    prices: UnorderedMap<AccountId, Price>,

    /// total orders created on contract
    order_nonce: u64,

    /// user ➝ order_id ➝ Order
    orders: UnorderedMap<AccountId, HashMap<u64, Order>>,

    /// (AccountId, AccountId) ➝ TradePair
    supported_markets: UnorderedMap<(AccountId, AccountId), TradePair>,

    /// User ➝ Token ➝ Balance
    balances: UnorderedMap<AccountId, HashMap<AccountId, Balance>>,
}

impl Default for Contract {
    fn default() -> Self {
        env::panic_str("Margin trading contract should be initialized before usage")
    }
}

#[near_bindgen]
impl Contract {
    #[init]
    #[private]
    pub fn new() -> Self {
        require!(!env::state_exists(), "Already initialized");

        Self {
            market_infos: LookupMap::new(StorageKeys::Markets),
            protocol_fee: 10u128.pow(24),
            prices: UnorderedMap::new(StorageKeys::Prices),
            order_nonce: 0,
            orders: UnorderedMap::new(StorageKeys::Orders),
            supported_markets: UnorderedMap::new(StorageKeys::SupportedMarkets),
            balances: UnorderedMap::new(StorageKeys::Balances),
        }
    }

    #[private]
    pub fn add_market_data(&mut self, market: AccountId, data: MarketData) {
        self.market_infos.insert(&market, &data);
    }

    #[private]
    fn set_protocol_fee(&mut self, fee: U128) {
        self.protocol_fee = fee.0
    }
}
