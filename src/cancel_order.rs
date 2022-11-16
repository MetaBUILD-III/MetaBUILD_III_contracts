use crate::big_decimal::{BigDecimal, WRatio};
use crate::ref_finance::ext_ref_finance;
use crate::ref_finance::{Action, SwapAction, TokenReceiverMessage};
use crate::utils::NO_DEPOSIT;
use crate::utils::{ext_market, ext_token};
use crate::*;
use near_sdk::env::{block_height, current_account_id, signer_account_id};
use near_sdk::{ext_contract, is_promise_success, log, Gas, PromiseResult, ONE_YOCTO};

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn remove_liquidity_callback(
        &self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
    );
    fn order_cancel_swap_callback(
        &self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
    );
    fn market_data_callback(
        &self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
    );
    fn get_pool_callback(
        &self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
    );
    fn get_liquidity_callback(
        &self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
        pool_info: PoolInfo,
    );
    fn repay_callback(&self) -> PromiseOrValue<U128>;
}

#[near_bindgen]
impl Contract {
    pub fn cancel_order(&mut self, order_id: U128, swap_fee: U128, price_impact: U128) {
        let orders = self.orders.get(&signer_account_id()).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", signer_account_id());
        });

        let order = orders
            .get(&(order_id.0 as u64))
            .unwrap_or_else(|| {
                panic!("Order with id: {} not found", order_id.0);
            })
            .clone();
        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_unused_gas_weight(1)
            .with_attached_deposit(NO_DEPOSIT)
            .get_pool(self.view_pair(&order.sell_token, &order.buy_token).pool_id)
            .then(
                ext_self::ext(current_account_id())
                    .with_unused_gas_weight(29)
                    .with_attached_deposit(NO_DEPOSIT)
                    .get_pool_callback(
                        order_id,
                        order,
                        swap_fee,
                        price_impact,
                        OrderAction::Cancel,
                    ),
            );
    }

    #[private]
    pub fn get_pool_callback(
        &mut self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
    ) {
        require!(
            is_promise_success(),
            "Some problem with pool on ref finance"
        );
        let pool_info = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(pool) = near_sdk::serde_json::from_slice::<PoolInfo>(&val) {
                    pool
                } else {
                    panic!("Some problem with pool parsing.")
                }
            }
            PromiseResult::Failed => panic!("Ref finance not found pool"),
        };

        require!(
            pool_info.state == PoolState::Running,
            "Some problem with pool, please contact with ref finance to support."
        );

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_unused_gas_weight(2)
            .with_attached_deposit(NO_DEPOSIT)
            .get_liquidity(order.lpt_id.clone())
            .then(
                ext_self::ext(current_account_id())
                    .with_unused_gas_weight(98)
                    .with_attached_deposit(NO_DEPOSIT)
                    .get_liquidity_callback(
                        order_id,
                        order,
                        swap_fee,
                        price_impact,
                        order_action,
                        pool_info,
                    ),
            );
    }

    #[private]
    pub fn get_liquidity_callback(
        &mut self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
        pool_info: PoolInfo,
    ) {
        require!(
            is_promise_success(),
            "Some problem with liquidity on ref finance"
        );
        let liquidity: Liquidity = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(pool) = near_sdk::serde_json::from_slice::<Liquidity>(&val) {
                    pool
                } else {
                    panic!("Some problem with liquidity parsing.")
                }
            }
            PromiseResult::Failed => panic!("Ref finance not found liquidity"),
        };

        let remove_liquidity_amount = liquidity.amount.0;
        let min_amount_x = liquidity.amount.0 - 1000;
        let min_amount_y = 0;

        require!(
            pool_info.total_x.0 > remove_liquidity_amount,
            "Pool not have enough liquidity"
        );

        if order.status == OrderStatus::Pending {
            ext_ref_finance::ext(self.ref_finance_account.clone())
                .with_unused_gas_weight(50)
                .with_attached_deposit(NO_DEPOSIT)
                .remove_liquidity(
                    order.lpt_id.to_string(),
                    U128(remove_liquidity_amount),
                    U128(min_amount_x),
                    U128(min_amount_y),
                )
                .then(
                    ext_self::ext(current_account_id())
                        .with_unused_gas_weight(50)
                        .with_attached_deposit(NO_DEPOSIT)
                        .remove_liquidity_callback(
                            order_id,
                            order,
                            swap_fee,
                            price_impact,
                            order_action,
                        ),
                );
        } else {
            self.swap(order_id, order, swap_fee, price_impact, order_action);
        }
    }

    #[private]
    pub fn remove_liquidity_callback(
        &mut self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
    ) {
        require!(is_promise_success(), "Some problem with remove liquidity");
        self.order_cancel_swap_callback(order_id, order, swap_fee, price_impact, order_action);
    }

    pub fn swap(
        &self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
    ) {
        let buy_amount =
            BigDecimal::from(order.amount) * order.leverage * order.sell_token_price.value
                / order.buy_token_price.value;
        let min_amount = buy_amount * self.get_price(order.buy_token.clone());
        let actions: Vec<Action> = vec![Action::Swap(SwapAction {
            pool_id: self.view_pair(&order.sell_token, &order.buy_token).pool_id,
            token_in: order.buy_token.clone(),
            amount_in: Some(WRatio::from(200)),
            token_out: order.sell_token.clone(),
            min_amount_out: WRatio::from(200),
        })];
        let action = TokenReceiverMessage::Execute {
            force: true,
            actions,
        };

        log!(
            "action {}",
            near_sdk::serde_json::to_string(&action).unwrap()
        );

        ext_token::ext(order.buy_token.clone())
            .with_static_gas(Gas(3))
            .with_attached_deposit(1)
            .ft_transfer_call(
                self.ref_finance_account.clone(),
                WRatio::from(buy_amount),
                Some("Swap".to_string()),
                near_sdk::serde_json::to_string(&action).unwrap(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(20))
                    .with_attached_deposit(NO_DEPOSIT)
                    .order_cancel_swap_callback(
                        order_id,
                        order,
                        swap_fee,
                        price_impact,
                        order_action,
                    ),
            );
    }

    #[private]
    pub fn order_cancel_swap_callback(
        &mut self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
    ) {
        log!(
            "Order cancel swap callback attached gas: {}",
            env::prepaid_gas().0
        );
        let market_id = self.tokens_markets.get(&order.sell_token).unwrap();

        ext_market::ext(market_id)
            .with_static_gas(Gas(20))
            .with_attached_deposit(NO_DEPOSIT)
            .view_market_data()
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(70))
                    .with_attached_deposit(NO_DEPOSIT)
                    .market_data_callback(order_id, order, swap_fee, price_impact, order_action),
            );
    }

    #[private]
    pub fn market_data_callback(
        &mut self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
    ) {
        log!(
            "Market data callback attached gas: {}",
            env::prepaid_gas().0
        );
        require!(is_promise_success(), "failed to get market data.");
        let market_data = match env::promise_result(0) {
            PromiseResult::NotReady => panic!("failed to get market data"),
            PromiseResult::Successful(val) => {
                if let Ok(data) = near_sdk::serde_json::from_slice::<MarketData>(&val) {
                    data
                } else {
                    panic!("failed parse market data")
                }
            }
            PromiseResult::Failed => panic!("failed to get market data"),
        };

        if order_action == OrderAction::Cancel {
            self.final_order_cancel(order_id, order, market_data, swap_fee, price_impact)
        } else {
            self.final_liquidate(order_id, order, market_data);
        }
    }

    fn final_order_cancel(
        &mut self,
        order_id: U128,
        order: Order,
        market_data: MarketData,
        swap_fee: U128,
        price_impact: U128,
    ) {
        log!("Final order cancel attached gas: {}", env::prepaid_gas().0);

        let mut order = order.clone();
        let sell_amount =
            order.sell_token_price.value * BigDecimal::from(order.amount) * order.leverage;

        let pnl = self.calculate_pnl(signer_account_id(), order_id, market_data);

        let expect_amount = self.get_price(order.buy_token.clone())
            / BigDecimal::from(10_u128.pow(24))
            * sell_amount
            * (BigDecimal::from(1) - BigDecimal::from(swap_fee))
            * (BigDecimal::from(1) - BigDecimal::from(price_impact))
            / (order.buy_token_price.value / BigDecimal::from(10_u128.pow(24)));

        self.increase_balance(
            &signer_account_id(),
            &order.sell_token,
            expect_amount.round_u128(),
        );

        if pnl.is_profit && expect_amount > sell_amount + BigDecimal::from(pnl.amount) {
            let protocol_profit = expect_amount - sell_amount - BigDecimal::from(pnl.amount);

            let token_profit = self
                .protocol_profit
                .get(&order.sell_token)
                .unwrap_or_default();
            self.protocol_profit.insert(
                &order.sell_token,
                &(BigDecimal::from(token_profit) + protocol_profit),
            );
        }

        let mut orders = self.orders.get(&signer_account_id()).unwrap();
        order.status = OrderStatus::Canceled;
        orders.insert(order_id.0 as u64, order);
        self.orders.insert(&signer_account_id(), &orders);
    }

    pub fn repay(&self, order: Order, market_data: MarketData) {
        let market_id = self.tokens_markets.get(&order.sell_token).unwrap();
        let borrow_fee = BigDecimal::from(market_data.borrow_rate_ratio.0)
            * BigDecimal::from((block_height() - order.block) as u128);

        ext_token::ext(order.sell_token.clone())
            .with_static_gas(Gas::ONE_TERA * 35u64)
            .with_attached_deposit(ONE_YOCTO)
            .ft_transfer_call(
                market_id,
                U128(borrow_fee.round_u128()),
                None,
                "\"Repay\"".to_string(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 3u64)
                    .with_attached_deposit(NO_DEPOSIT)
                    .repay_callback(),
            );
    }

    #[private]
    pub fn repay_callback(&self) -> PromiseOrValue<U128> {
        require!(is_promise_success(), "failed to repay assets");
        //TODO: add repay success event
        PromiseOrValue::Value(U128(0))
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
    fn test_order_was_canceled() {
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

        let order1 = "{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000000000000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"block\":103930916,\"lpt_id\":\"usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#543\"}".to_string();
        contract.add_order(alice(), order1.clone());

        let order_id = U128(1);
        let order = Order {
            status: OrderStatus::Pending,
            order_type: OrderType::Buy,
            amount: 1000000000000000000000000000,
            sell_token: "usdt.qa.v1.nearlend.testnet".parse().unwrap(),
            buy_token: "wnear.qa.v1.nearlend.testnet".parse().unwrap(),
            leverage: BigDecimal::from(1.0),
            sell_token_price: Price {
                ticker_id: "USDT".to_string(),
                value: BigDecimal::from(1.01),
            },
            buy_token_price: Price {
                ticker_id: "near".to_string(),
                value: BigDecimal::from(3.07),
            },
            block: 105210654,
            lpt_id: "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#238".to_string(),
        };

        let market_data = MarketData {
            total_supplies: U128(60000000000000000000000000000),
            total_borrows: U128(25010000000000000000000000000),
            total_reserves: U128(1000176731435219096024128768),
            exchange_rate_ratio: U128(1000277139994639276176632),
            interest_rate_ratio: U128(261670051778601),
            borrow_rate_ratio: U128(634273735391536),
        };

        let swap_fee = U128(1);
        let price_impact = U128(1);
        contract.final_order_cancel(order_id, order, market_data, swap_fee, price_impact);

        let orders = contract.orders.get(&alice()).unwrap();
        let order = orders.get(&1).unwrap();
        assert_eq!(order.status, OrderStatus::Canceled);
    }
}
