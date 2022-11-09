use crate::big_decimal::{BigDecimal, WRatio};
use crate::*;
use near_sdk::env::block_height;

#[near_bindgen]
impl Contract {
    pub fn view_market_data(&self, market: AccountId) -> MarketData {
        self.market_infos.get(&market).unwrap_or_default()
    }

    pub fn view_order(&self, account_id: AccountId, order_id: U128) -> OrderView {
        let orders = self.orders.get(&account_id).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", account_id);
        });

        let order = orders
            .get(&(order_id.0 as u64))
            .unwrap_or_else(|| {
                panic!("Order with id: {} not found", order_id.0);
            })
            .clone();

        OrderView {
            order_id,
            status: order.status,
            order_type: order.order_type,
            amount: U128(order.amount),
            sell_token: order.sell_token,
            buy_token: order.buy_token,
            buy_token_price: WBalance::from(order.buy_token_price.value),
            fee: U128(3 * 10u128.pow(23)), // hardcore of 0.3 %
        }
    }

    pub fn calculate_pnl(
        &self,
        account_id: AccountId,
        order_id: U128,
        data: MarketData,
    ) -> PnLView {
        let orders = self.orders.get(&account_id).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", account_id);
        });

        let order = orders
            .get(&(order_id.0 as u64))
            .unwrap_or_else(|| {
                panic!("Order with id: {} not found", order_id.0);
            })
            .clone();

        let sell_amount_open =
            BigDecimal::from(order.amount) * order.leverage * order.sell_token_price.value;

        let borrow_amount =
            BigDecimal::from(order.amount) * order.leverage - BigDecimal::from(order.amount);

        let expect_amount = (borrow_amount / self.get_price(order.buy_token.clone())
            + BigDecimal::from(order.amount))
            * self.get_price(order.sell_token.clone());

        let borrow_fee =
            BigDecimal::from(data.borrow_rate_ratio.0 * (block_height() - order.block) as u128);

        let pnlv: PnLView = if expect_amount > sell_amount_open + borrow_fee {
            let lenpnl = (expect_amount - sell_amount_open - borrow_fee)
                * BigDecimal::from(10_u128.pow(24) - self.protocol_fee);
            PnLView {
                is_profit: true,
                amount: WRatio::from(lenpnl),
            }
        } else {
            let lenpnl = (sell_amount_open + borrow_fee - expect_amount)
                * BigDecimal::from(10_u128.pow(24) - self.protocol_fee);
            PnLView {
                is_profit: false,
                amount: WRatio::from(lenpnl),
            }
        };

        pnlv
    }

    pub fn view_orders(
        &self,
        account_id: AccountId,
        sell_token: AccountId,
        buy_token: AccountId,
    ) -> Vec<OrderView> {
        let orders = self.orders.get(&account_id).unwrap_or_default();
        let result = orders
            .iter()
            .filter_map(|(id, order)| {
                match order.sell_token == sell_token && order.buy_token == buy_token {
                    true => Some(OrderView {
                        order_id: U128(*id as u128),
                        status: order.status.clone(),
                        order_type: order.order_type.clone(),
                        amount: U128(order.amount.clone()),
                        sell_token: order.sell_token.clone(),
                        buy_token: order.buy_token.clone(),
                        buy_token_price: WRatio::from(order.buy_token_price.value),
                        fee: U128(self.protocol_fee),
                    }),
                    false => None,
                }
            })
            .collect::<Vec<OrderView>>();
        result
    }

    pub fn view_pair(&self, sell_token: AccountId, buy_token: AccountId) -> TradePair {
        self.supported_markets
            .get(&(sell_token, buy_token))
            .unwrap()
    }

    pub fn view_supported_pairs(&self) -> Vec<TradePair> {
        let pairs = self
            .supported_markets
            .iter()
            .map(|(_, trade_pair)| trade_pair)
            .collect::<Vec<TradePair>>();

        pairs
    }

    pub fn balance_of(&self, account_id: AccountId, token: AccountId) -> Balance {
        match self.balances.get(&account_id) {
            None => 0,
            Some(user_balance_per_token) => *user_balance_per_token.get(&token).unwrap_or(&0u128),
        }
    }

    pub fn view_price(&self, token_id: AccountId) -> Price {
        self.prices.get(&token_id).unwrap_or_else(|| {
            panic!("Price for token: {} not found", token_id);
        })
    }

    pub fn cancel_order_view(
        &self,
        account_id: AccountId,
        order_id: U128,
        market_data: MarketData,
    ) -> CancelOrderView {
        let orders = self.orders.get(&account_id).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", account_id);
        });

        let order = orders.get(&(order_id.0 as u64)).unwrap_or_else(|| {
            panic!("Order with id: {} not found", order_id.0);
        });

        let buy_token =
            BigDecimal::from(order.amount) * order.leverage * order.sell_token_price.value
                / order.buy_token_price.value;

        let sell_token = BigDecimal::from(order.amount) * order.leverage;

        let open_price = order.buy_token_price.clone();

        let close_price = self.get_price(order.buy_token.clone());

        let calc_pnl = self.calculate_pnl(account_id, order_id, market_data);

        CancelOrderView {
            buy_token_amount: WRatio::from(buy_token),
            sell_token_amount: WRatio::from(sell_token),
            open_price: WRatio::from(open_price.value),
            close_price: WRatio::from(close_price),
            pnl: calc_pnl,
        }
    }

    pub fn view_liquidation_threshold(&self) -> U128 {
        U128(self.liquidation_threshold)
    }

    pub fn calculate_liquidation_price(
        &self,
        sell_token_amount: U128,
        sell_token_price: U128,
        buy_token_price: U128,
        leverage: U128,
        borrow_fee: U128,
        swap_fee: U128,
    ) -> Price {
        let volatility_rate = BigDecimal::from(U128(95 * 10_u128.pow(22)));
        let collateral_usd =
            BigDecimal::from(sell_token_amount) * BigDecimal::from(sell_token_price);
        let position_amount_usd = collateral_usd * BigDecimal::from(leverage.0);
        let borrow_amount = collateral_usd * (BigDecimal::from(leverage.0) - BigDecimal::from(1));
        let buy_amount = position_amount_usd / BigDecimal::from(buy_token_price);

        let liquidation_price = (position_amount_usd - volatility_rate * collateral_usd
            + borrow_amount * BigDecimal::from(borrow_fee)
            + position_amount_usd * BigDecimal::from(swap_fee))
            / buy_amount;

        Price {
            ticker_id: "usd".to_string(),
            value: liquidation_price,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk::test_utils::test_env::alice;
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{serde_json, testing_env, FunctionError, VMContext};

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .current_account_id("margin.nearland.testnet".parse().unwrap())
            .signer_account_id(alice())
            .predecessor_account_id("usdt_market.qa.nearland.testnet".parse().unwrap())
            .block_index(103930920)
            .block_timestamp(1)
            .is_view(is_view)
            .build()
    }

    #[test]
    fn view_supported_pairs_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );
        let pair_data = TradePair {
            sell_ticker_id: "usdt".to_string(),
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_market: "usdt_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "wnear".to_string(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
        };
        contract.add_pair(pair_data.clone());

        let pair_data2 = TradePair {
            sell_ticker_id: "wnear".to_string(),
            sell_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            sell_token_market: "wnear_market.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_ticker_id: "usdt".to_string(),
            buy_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
        };

        contract.add_pair(pair_data2.clone());

        let result = vec![pair_data, pair_data2];
        let pairs = contract.view_supported_pairs();
        assert_eq!(result, pairs);
    }

    #[test]
    fn view_orders_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        let order1 = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":1000000100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"block\":103930916,\"lpt_id\":\"1\"}".to_string();
        contract.add_order(alice(), order1.clone());

        let order2 = "{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000001100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"0.99\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.99\"},\"block\":103930918,\"lpt_id\":\"3\"}".to_string();
        contract.add_order(alice(), order2.clone());

        let orders = contract.view_orders(
            alice(),
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
        );
        let order1_un: Order = serde_json::from_str(order1.as_str()).unwrap();
        let order2_un: Order = serde_json::from_str(order2.as_str()).unwrap();
        let expect_result = vec![
            OrderView {
                order_id: U128(1),
                status: order1_un.status,
                order_type: order1_un.order_type,
                amount: U128(order1_un.amount),
                sell_token: order1_un.sell_token,
                buy_token: order1_un.buy_token,
                buy_token_price: WRatio::from(order1_un.buy_token_price.value),
                fee: U128(contract.protocol_fee),
            },
            OrderView {
                order_id: U128(2),
                status: order2_un.status,
                order_type: order2_un.order_type,
                amount: U128(order2_un.amount),
                sell_token: order2_un.sell_token,
                buy_token: order2_un.buy_token,
                buy_token_price: WRatio::from(order2_un.buy_token_price.value),
                fee: U128(contract.protocol_fee),
            },
        ];
        assert_eq!(expect_result, orders);
    }

    #[test]
    fn calculate_pnl_test() {
        let context = get_context(false);
        testing_env!(context);
        let mut contract = Contract::new_with_config(
            "owner_id.testnet".parse().unwrap(),
            "oracle_account_id.testnet".parse().unwrap(),
        );

        contract.update_or_insert_price(
            "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "USDT".to_string(),
                value: BigDecimal::from(2.0),
            },
        );
        contract.update_or_insert_price(
            "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            Price {
                ticker_id: "WNEAR".to_string(),
                value: BigDecimal::from(4.22),
            },
        );
        let order1 = "{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":1000000100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"block\":103930916,\"lpt_id\":\"1\"}".to_string();
        contract.add_order(alice(), order1.clone());
        let market_data = MarketData {
            total_supplies: U128(10_u128.pow(24)),
            total_borrows: U128(10_u128.pow(24)),
            total_reserves: U128(10_u128.pow(24)),
            exchange_rate_ratio: U128(10_u128.pow(24)),
            interest_rate_ratio: U128(10_u128.pow(24)),
            borrow_rate_ratio: U128(10_u128.pow(24)),
        };
        let pnl = contract.calculate_pnl(alice(), U128(1), market_data);
        assert!(pnl.is_profit);
        assert_eq!(pnl.amount, U128(256558246105184350685977637895071072256));
    }
}
