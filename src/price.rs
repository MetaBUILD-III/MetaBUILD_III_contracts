use crate::big_decimal::{BigDecimal, WBalance};
use crate::create_order::Action;
use crate::*;
use std::str::FromStr;

#[near_bindgen]
impl Contract {
    #[private]
    pub fn update_or_insert_price(&mut self, token_id: AccountId, price: Price) {
        self.prices.insert(&token_id, &price);
    }

    pub fn view_price(&self, token_id: AccountId) -> Price {
        self.prices.get(&token_id).unwrap_or_else(|| {
            panic!("Price for token: {} not found", token_id);
        })
    }

    pub fn calculate_xrate(&self, token_id_1: AccountId, token_id_2: AccountId) -> BigDecimal {
        self.view_price(token_id_1).value / self.view_price(token_id_2).value
    }

    pub fn get_market_by_token(&self, token: AccountId) -> Option<AccountId> {
        let mut result = vec![];
        for trade_pair in self.view_supported_pairs().iter() {
            if trade_pair.sell_token == token {
                result.push(trade_pair.sell_token_market.clone());
            }
        }

        result.pop()
    }
}
