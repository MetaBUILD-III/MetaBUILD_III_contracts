use crate::big_decimal::BigDecimal;
use crate::*;

#[near_bindgen]
impl Contract {
    #[private]
    pub fn update_or_insert_price(&mut self, token_id: AccountId, price: Price) {
        self.prices.insert(&token_id, &price);
    }

    pub fn get_price(&self, token_id: AccountId) -> BigDecimal {
        self.prices
            .get(&token_id)
            .unwrap_or_else(|| {
                panic!("Price for token: {} not found", token_id);
            })
            .value
    }

    pub fn calculate_xrate(&self, token_id_1: AccountId, token_id_2: AccountId) -> BigDecimal {
        self.view_price(token_id_1).value / self.view_price(token_id_2).value
    }

    pub fn get_market_by(&self, token: AccountId) -> AccountId {
        self.tokens_markets.get(&token).unwrap_or_else(|| {
            panic!("Market for token: {} was not found", token);
        })
    }
}
