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

    pub fn get_liquidation_price(
        &self,
        sell_token_amount: U128,
        sell_token_price: U128,
        buy_token_price: U128,
        leverage: U128,
        borrow_fee: U128,
        swap_fee: U128,
    ) -> WBigDecimal {

        let collateral_usd = BigDecimal::from(sell_token_amount) * BigDecimal::from(sell_token_price);
        
        let buy_amount = collateral_usd / BigDecimal::from(buy_token_price);

        let borrow_amount = collateral_usd * BigDecimal::from(leverage);

        (BigDecimal::from(buy_token_price) - (collateral_usd - BigDecimal::from(borrow_fee) * borrow_amount) / buy_amount + BigDecimal::from(sell_token_amount) * BigDecimal::from(swap_fee)).into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_liquidation_price() {
        
        let owner_id: AccountId = "limit_orders.v1.nearlend.testnet".parse().unwrap();
        let oracle_account_id: AccountId = "limit_orders_oracle.v1.nearlend.testnet".parse().unwrap();
        
        let contract = Contract::new_with_config(owner_id, oracle_account_id);

        let result = contract.get_liquidation_price(          
            U128(1000000000000000000000000), 
            U128(1000999999999999900000000), 
            U128(2912000000000000000000000),
            U128(1),
            U128(5073566717402330000000000),
            U128(3000000000000000000000),
        );

        assert_eq!(result, U128(3000000000000000000000));
    }
}