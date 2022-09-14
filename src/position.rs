
use crate::*;
use std::fmt;

use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance};

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct ViewPosition {
    pub active: bool,
    pub position_id: U128,
    pub p_type: PositionType,
    pub amount: WBalance,
    pub price: WBalance,
    pub fee: U128,
    pub sell_token: AccountId,
    pub buy_token: AccountId,
}

impl fmt::Display for ViewPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Position {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        position_id: u128,
        active: bool,
        p_type: PositionType,
        sell_token: AccountId,
        buy_token: AccountId,
        collateral_amount: Balance,
        buy_token_price: Balance,
        sell_token_price: Balance,
        leverage: u128,
        borrow_amount: Balance,
    ) -> Position {
        Position {
            position_id,
            active,
            p_type,
            sell_token,
            buy_token,
            collateral_amount,
            buy_token_price,
            sell_token_price,
            leverage,
            borrow_amount,
        }
    }
}
