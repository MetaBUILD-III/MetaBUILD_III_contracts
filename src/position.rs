use crate::ratio::Ratio;
use crate::*;
use std::fmt;

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
