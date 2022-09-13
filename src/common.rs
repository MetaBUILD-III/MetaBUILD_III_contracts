use crate::utils::Digits;
use crate::*;
use near_sdk::Balance;
use std::fmt;
use std::fmt::Formatter;

pub enum Events {
    BorrowFailedOnMarket(Balance),
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug)]
pub struct Price {
    /// Ticker price value
    pub value: WBalance,

    /// Ticker precision digits number
    pub fraction_digits: Digits,
}

impl Price {
    pub fn new(value: Balance, fraction_digits: u32) -> Price {
        Price {
            value: WBalance::from(value),
            fraction_digits,
        }
    }
}

impl fmt::Display for Events {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Events::BorrowFailedOnMarket(balance) => {
                write!(
                    f,
                    r#"EVENT_JSON:{{"standard": "nep297", "version": "1.0.0", "event": "BorrowFailedOnMarket", "data": {{"reason": "failed to get {} borrow on market "}}}}"#,
                    balance
                )
            }
        }
    }
}
