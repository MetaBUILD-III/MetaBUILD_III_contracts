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

#[near_bindgen]
impl Contract {
    pub fn calculate_liquidation_price(
        sell_token_amount: U128,
        sell_token_price: Price,
        buy_token_price: Price,
        leverage: U128,
        borrow_fee: U128,
        swap_fee: U128,
    ) -> Price {
        let volatility_rate = BigDecimal::from(U128(950000000000000000000000));

        let collateral_usd = BigDecimal::from(sell_token_amount ) * BigDecimal::from(sell_token_price.value);
        let position_amount_usd = collateral_usd * BigDecimal::from(leverage.0);
        let borrow_amount = collateral_usd * (BigDecimal::from(leverage.0) - BigDecimal::from(1));
        let buy_amount = position_amount_usd / BigDecimal::from(buy_token_price.value);

        let liquidation_price =
        (position_amount_usd
        - volatility_rate
        * collateral_usd
        + borrow_amount
        * BigDecimal::from(borrow_fee)
        + position_amount_usd
        * BigDecimal::from(swap_fee))
        / buy_amount;
        
        Price{ 
            ticker_id: "usd".to_string(),
            value: liquidation_price,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_liquidation_price_sell_usdt() {
        
        let sell_token_price = Price {
            ticker_id: "usdt".to_string(),
            value: BigDecimal::from(U128(10_u128.pow(24)))
        };

        let buy_token_price = Price {
            ticker_id: "wnear".to_string(),
            value: BigDecimal::from(U128(10_u128.pow(25)))
        };

        let result = Contract::calculate_liquidation_price(          
            U128(10_u128.pow(27)), 
            sell_token_price, 
            buy_token_price,
            U128(3),
            U128(5 * 10_u128.pow(22)),
            U128(3 * 10_u128.pow(20)),
        );

        assert_eq!(
            (result.ticker_id, result.value),
            ("usd".to_string(), BigDecimal::from(U128(7169666666666666666666666)))
        );
    }

    #[test]
    fn test_calculate_liquidation_price_sell_wnear() {
        
        let sell_token_price = Price {
            ticker_id: "wnear".to_string(),
            value: BigDecimal::from(U128(10_u128.pow(25)))
        };

        let buy_token_price = Price {
            ticker_id: "usdt".to_string(),
            value: BigDecimal::from(U128(10_u128.pow(24)))
        };

        let result = Contract::calculate_liquidation_price(          
            U128(10_u128.pow(27)), 
            sell_token_price, 
            buy_token_price,
            U128(2),
            U128(5 * 10_u128.pow(22)),
            U128(3 * 10_u128.pow(20)),
        );

        assert_eq!(
            (result.ticker_id, result.value),
            ("usd".to_string(), BigDecimal::from(U128(550300000000000000000000)))
        );
    }
}