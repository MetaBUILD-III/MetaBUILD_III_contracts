use crate::big_decimal::BigDecimal;
use crate::*;
use std::str::FromStr;
use near_sdk::{log};

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

    #[allow(unused_variables)]
    pub fn get_liquidation_price(
        &self,
        sell_token_amount: U128,
        sell_token_price: U128,
        buy_token_price: U128,
        leverage: U128,
        borrow_fee: U128,
        swap_fee: U128,
    ) -> WBigDecimal {
        let sell_token = AccountId::new_unchecked("usdt.qa.v1.nearlend.testnet".to_owned());
        let sell_token_price = self.get_price(sell_token);

        let buy_token = AccountId::new_unchecked("wnear.qa.v1.nearlend.testnet".to_owned());
        let buy_token_price = self.get_price(buy_token);
        log!("buy_token_price {}", buy_token_price.0);

        let collateral_usd = BigDecimal::from(sell_token_amount) * BigDecimal::from(sell_token_price);
        let buy_amount = collateral_usd / BigDecimal::from(buy_token_price);

        let fee = BigDecimal::from_str("0.057").unwrap();
        let borrow_amount = collateral_usd * BigDecimal::from(leverage);

        (BigDecimal::from(buy_token_price) -  (collateral_usd - fee * borrow_amount) / buy_amount).into()
    }
}
