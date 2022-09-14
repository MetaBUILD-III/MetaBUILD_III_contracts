use crate::utils::Digits;
use crate::*;

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

#[near_bindgen]
impl Contract {
    pub fn set_price(&mut self, market_id: AccountId, price: Price) {
        self.prices.insert(&market_id, &price);
    }

    pub fn get_price_by_token(&self, token_id: AccountId) -> WBalance {
        assert!(
            self.prices.get(&token_id).is_some(),
            "There no such prices set yet"
        );

        self.prices.get(&token_id).unwrap().value
    }

    pub fn calculate_xrate(&self, token_id_1: AccountId, token_id_2: AccountId) -> Ratio {
        Ratio::from(self.get_price_by_token(token_id_1))
            / Ratio::from(self.get_price_by_token(token_id_2))
    }
}
