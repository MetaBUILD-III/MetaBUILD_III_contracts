use crate::*;
use crate::big_decimal::{BigDecimal, WBalance};

#[near_bindgen]
impl Contract {
    #[private]
    pub fn update_or_insert_price(&mut self, token_id: AccountId, price: Price) {
        self.prices.insert(&token_id, &price);
    }

    #[private]
    pub fn view_price(&self, token_id: AccountId) -> Price {
        self.prices.get(&token_id).unwrap_or_else(|| {
            panic!("Price for token: {} not found", token_id);
        })
    }

    pub fn calculate_xrate(&self, token_id_1: AccountId, token_id_2: AccountId) -> BigDecimal {
        self.view_price(token_id_1).value
            / self.view_price(token_id_2).value
    }
}
