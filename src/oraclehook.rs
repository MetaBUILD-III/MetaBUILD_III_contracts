use crate::*;
use near_sdk::serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct PriceJsonList {
    /// Block number
    pub block_height: u64,

    /// Vector of asset prices
    pub price_list: Vec<Price>,
}

pub trait OraclePriceHandlerHook {
    fn oracle_on_data(&mut self, price_data: PriceJsonList);
}

#[near_bindgen]
impl OraclePriceHandlerHook for Contract {
    fn oracle_on_data(&mut self, price_data: PriceJsonList) {
        let config: Config = self.get_contract_config();

        assert_eq!(
            env::predecessor_account_id(),
            config.oracle_account_id,
            "Oracle account {} doesn't match to the signer {}",
            config.oracle_account_id,
            env::predecessor_account_id()
        );


        let ticker_map = self.get_ticker_map();


        for price in price_data.price_list {
            if let Some(token) = ticker_map.get(&price.ticker_id) {
                self.update_or_insert_price(token.clone(), price.clone())
            }
        }
    }
}

impl Contract {
    pub fn get_ticker_map(&mut self) -> HashMap<String, AccountId> {
        let mut ticker_map = HashMap::new();
        self.supported_markets.values().for_each(|trade_pair| {
            ticker_map.insert(trade_pair.sell_ticker_id, trade_pair.sell_token);
            ticker_map.insert(trade_pair.buy_ticker_id, trade_pair.buy_token);
        });
        ticker_map
    }
}
