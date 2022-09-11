use near_sdk::Balance;
use std::fmt;
use std::fmt::Formatter;

pub enum Events {
    BorrowFailedOnMarket(Balance),
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
