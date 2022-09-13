use crate::ratio::Ratio;
use crate::*;
use std::fmt;

use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{AccountId, Balance};

pub const REF_FINANCE: &str = "ref-finance-101.testnet";

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct ViewPosition {
    position_id: U128,
    p_type: PositionType,
    amount: WBalance,
    price: WBalance,
    fee: U128,
}

impl ViewPosition {
    pub fn new(position_id: u128, amount: Balance, price: Balance, fee: Ratio) -> ViewPosition {
        ViewPosition {
            position_id: U128::from(position_id),
            p_type: PositionType::Long,
            amount: U128::from(amount),
            price: U128::from(price),
            fee: U128::from(fee),
        }
    }
}

impl fmt::Display for ViewPosition {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Position {
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
        }
    }
}