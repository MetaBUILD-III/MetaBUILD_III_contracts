use crate::big_decimal::{BigDecimal, WBalance, WRatio};
use crate::*;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{Balance, BlockHeight, BorshStorageKey};

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    Markets,
    Prices,
    Orders,
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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct PnLView {
    pub(crate) is_profit: bool,
    pub(crate) amount: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct Price {
    ticker_id: String,
    value: BigDecimal,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum OrderStatus {
    Pending,
    Executed,
    Canceled,
    Liquidated,
}

#[derive(BorshDeserialize, BorshSerialize)]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(BorshDeserialize, BorshSerialize)]
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

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OrderView {
    pub(crate) order_id: U128,
    pub(crate) status: OrderStatus,
    pub(crate) order_type: OrderType,
    pub(crate) amount: Balance,
    pub(crate) sell_token: AccountId,
    pub(crate) buy_token: AccountId,
    pub(crate) buy_token_price: WBalance,
    pub(crate) fee: WBalance,
}
