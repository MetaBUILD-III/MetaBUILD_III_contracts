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
    TokenMarkets,
    ProtocolProfit,
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

impl Default for MarketData {
    fn default() -> Self {
        Self {
            total_supplies: U128(0),
            total_borrows: U128(0),
            total_reserves: U128(0),
            exchange_rate_ratio: U128(0),
            interest_rate_ratio: U128(0),
            borrow_rate_ratio: U128(0),
        }
    }
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct PnLView {
    pub is_profit: bool,
    pub amount: U128,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Price {
    pub ticker_id: String,
    pub value: BigDecimal,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Clone, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum OrderStatus {
    Pending,
    Executed,
    Canceled,
    Liquidated,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub enum OrderType {
    Buy,
    Sell,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Order {
    pub status: OrderStatus,
    pub order_type: OrderType,
    pub amount: Balance,
    pub sell_token: AccountId,
    pub buy_token: AccountId,
    pub leverage: BigDecimal,
    pub sell_token_price: Price,
    pub buy_token_price: Price,
    pub block: BlockHeight,
    pub lpt_id: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct OrderView {
    pub order_id: U128,
    pub status: OrderStatus,
    pub order_type: OrderType,
    pub amount: U128,
    pub sell_token: AccountId,
    pub buy_token: AccountId,
    pub leverage: WBigDecimal,
    pub buy_token_price: WBalance,
    pub fee: WBalance,
    pub lpt_id: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug, Clone, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct TradePair {
    pub sell_ticker_id: String,
    pub sell_token: AccountId,
    pub sell_token_market: AccountId,
    pub buy_ticker_id: String,
    pub buy_token: AccountId,
    pub pool_id: String,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct CancelOrderView {
    pub buy_token_amount: WRatio,
    pub sell_token_amount: WRatio,
    pub open_price: WRatio,
    pub close_price: WRatio,
    pub pnl: PnLView,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum OrderAction {
    Create,
    Cancel,
    Liquidate,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub struct PoolInfo {
    pub pool_id: String,
    pub token_x: AccountId,
    pub token_y: AccountId,
    pub fee: u64,
    pub point_delta: u64,
    pub current_point: i64,
    pub liquidity: U128,
    pub liquidity_x: U128,
    pub max_liquidity_per_point: U128,
    pub volume_x_in: U128,
    pub volume_y_in: U128,
    pub volume_x_out: U128,
    pub volume_y_out: U128,
    pub total_liquidity: U128,
    pub total_order_x: U128,
    pub total_order_y: U128,
    pub total_x: U128,
    pub total_y: U128,
    pub state: PoolState,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, PartialEq)]
#[serde(crate = "near_sdk::serde")]
pub enum PoolState {
    Running,
}
