extern crate core;

mod big_decimal;
mod cancel_order;
mod common;
mod config;
mod create_order;
mod deposit;
mod execute_order;
mod ft;
mod liquidate_order;
mod market;
mod metadata;
mod oraclehook;
mod price;
mod ref_finance;
mod utils;
mod view;

use crate::big_decimal::*;
use crate::config::Config;
use crate::metadata::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, require, AccountId, Balance, PromiseOrValue};
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

    config: Config,

    /// Pool id in Ref Finance
    /// "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000"
    /// OR
    /// "wnear.qa.v1.nearlend.testnet|usdt.qa.v1.nearlend.testnet|2000"
    pool_id: String,

    /// token id -> market id
    tokens_markets: LookupMap<AccountId, AccountId>,

    /// Protocol profit token_id -> amount
    protocol_profit: LookupMap<AccountId, BigDecimal>,

    /// Ref finance accountId [ as default "dcl.ref-dev.testnet" ]
    ref_finance_account: AccountId,

    /// Liquidation threshold
    liquidation_threshold: u128,

    /// Volatility rate
    volatility_rate: BigDecimal,
}

impl Default for Contract {
    fn default() -> Self {
        env::panic_str("Margin trading contract should be initialized before usage")
    }
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract with the given config. Needs to be called once.
    #[init]
    pub fn new_with_config(owner_id: AccountId, oracle_account_id: AccountId) -> Self {
        Self::new(Config {
            owner_id,
            oracle_account_id,
        })
    }

    #[init]
    #[private]
    pub fn new(config: Config) -> Self {
        require!(!env::state_exists(), "Already initialized");

        Self {
            market_infos: LookupMap::new(StorageKeys::Markets),
            protocol_fee: 10u128.pow(23),
            prices: UnorderedMap::new(StorageKeys::Prices),
            order_nonce: 0,
            orders: UnorderedMap::new(StorageKeys::Orders),
            supported_markets: UnorderedMap::new(StorageKeys::SupportedMarkets),
            config,
            balances: UnorderedMap::new(StorageKeys::Balances),
            pool_id: "".to_string(),
            tokens_markets: LookupMap::new(StorageKeys::TokenMarkets),
            protocol_profit: LookupMap::new(StorageKeys::ProtocolProfit),
            ref_finance_account: "dcl.ref-dev.testnet".parse().unwrap(),
            liquidation_threshold: 10_u128.pow(23),
            volatility_rate: BigDecimal::from(U128(95 * 10_u128.pow(22))),
        }
    }

    #[private]
    pub fn add_market_data(&mut self, market: AccountId, data: MarketData) {
        self.market_infos.insert(&market, &data);
    }

    #[private]
    pub fn set_protocol_fee(&mut self, fee: U128) {
        self.protocol_fee = fee.0
    }

    #[private]
    pub fn add_token_market(&mut self, token_id: AccountId, market_id: AccountId) {
        self.tokens_markets.insert(&token_id, &market_id);
    }

    #[private]
    pub fn set_pool_id(&mut self, pool_id: String) {
        self.pool_id = pool_id;
    }

    #[private]
    pub fn set_liquidation_threshold(&mut self, threshold: U128) {
        self.liquidation_threshold = threshold.0;
    }

    #[private]
    pub fn set_volatility_rate(&mut self, rate: U128) {
        self.volatility_rate = BigDecimal::from(rate)
    }
}
