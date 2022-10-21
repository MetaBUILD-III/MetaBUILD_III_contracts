use crate::big_decimal::{BigDecimal, WBalance, WRatio};
use crate::*;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{Balance, BlockHeight, BorshStorageKey};

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    Markets,
    Prices,
    Orders,
    SupportedMarkets,
    Balances,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct MarketData {
    pub total_supplies: WBalance,
    pub total_borrows: WBalance,
    pub total_reserves: WBalance,
    pub exchange_rate_ratio: WRatio,
    pub interest_rate_ratio: WRatio,
    pub borrow_rate_ratio: WRatio,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PnLView {
    pub is_profit: bool,
    pub amount: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Price {
    pub ticker_id: String,
    value: BigDecimal,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum OrderStatus {
    Pending,
    Executed,
    Canceled,
    Liquidated,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Order {
    status: OrderStatus,
    order_type: OrderType,
    amount: Balance,
    sell_token: AccountId,
    buy_token: AccountId,
    leverage: BigDecimal,
    sell_token_price: Price,
    buy_token_price: Price,
    block: BlockHeight,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct OrderView {
    pub order_id: U128,
    pub status: OrderStatus,
    pub order_type: OrderType,
    pub amount: Balance,
    pub sell_token: AccountId,
    pub buy_token: AccountId,
    pub buy_token_price: WBalance,
    pub fee: WBalance,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct TradePair {
    pub sell_ticker_id: String,
    pub sell_token: AccountId,
    pub sell_token_market: AccountId,
    pub buy_ticker_id: String,
    pub buy_token: AccountId,
}
