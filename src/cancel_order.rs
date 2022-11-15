use crate::big_decimal::{BigDecimal, WRatio};
use crate::ref_finance::ext_ref_finance;
use crate::ref_finance::{Action, SwapAction, TokenReceiverMessage};
use crate::utils::NO_DEPOSIT;
use crate::utils::{ext_market, ext_token};
use crate::*;
use near_sdk::env::{current_account_id, signer_account_id};
use near_sdk::{ext_contract, is_promise_success, log, Gas, PromiseResult};

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
        &mut self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
    );
    fn market_data_callback(
        &mut self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
        market_data: Option<MarketData>,
    );
    fn cancel_order_callback(
        &mut self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
    );
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
            .with_static_gas(Gas(10))
            .with_attached_deposit(NO_DEPOSIT)
            .get_liquidity(order.lpt_id.clone())
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(5))
                    .with_attached_deposit(NO_DEPOSIT)
                    .cancel_order_callback(
                        order_id,
                        order,
                        swap_fee,
                        price_impact,
                        OrderAction::Cancel,
                    ),
            );
    }

    #[private]
    pub fn cancel_order_callback(
        &mut self,
        order_id: U128,
        order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
    ) {
        require!(is_promise_success(), "Failed to get_liquidity");
        let position = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                near_sdk::serde_json::from_slice::<crate::ref_finance::LiquidityInfo>(&val).unwrap()
            }
            PromiseResult::Failed => panic!("Ref finance not found pool"),
        };

        // require!(
        //     pool_info.state == PoolState::Running,
        //     "Some problem with pool, please contact with ref finance to support."
        // );

        let remove_liquidity_amount = position.amount;
        // TODO fix precision lost
        let min_amount_x = order.amount - 1000;
        let min_amount_y = 0;
        // require!(pool_info.total_x.0 < remove_liquidity_amount, "Pool not hav enough liquidity");

        if order.status == OrderStatus::Pending {
            ext_ref_finance::ext(self.ref_finance_account.clone())
                .with_static_gas(Gas(10))
                .with_attached_deposit(NO_DEPOSIT)
                .remove_liquidity(
                    order.lpt_id.to_string(),
                    remove_liquidity_amount,
                    U128(min_amount_x),
                    U128(min_amount_y),
                )
                .then(
                    ext_self::ext(current_account_id())
                        .with_static_gas(Gas(5))
                        .with_attached_deposit(NO_DEPOSIT)
                        .remove_liquidity_callback(
                            order_id,
                            order,
                            swap_fee,
                            price_impact,
                            OrderAction::Cancel,
                        ),
                );
        } else {
            self.swap(order_id, order, swap_fee, price_impact, OrderAction::Cancel);
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
        require!(is_promise_success(), "Some problem wish swap tokens");

        let market_id = self.tokens_markets.get(&order.sell_token).unwrap();
        if order.leverage > BigDecimal::from(10_u128.pow(24)) {
            ext_market::ext(market_id)
                .with_static_gas(Gas(7))
                .with_attached_deposit(1)
                .view_market_data()
                .then(
                    ext_self::ext(current_account_id())
                        .with_static_gas(Gas(3))
                        .with_attached_deposit(NO_DEPOSIT)
                        .market_data_callback(
                            order_id,
                            order,
                            swap_fee,
                            price_impact,
                            order_action,
                            None,
                        ),
                );
        } else {
            let market_data = self.market_infos.get(&market_id).unwrap();
            self.market_data_callback(
                order_id,
                order,
                swap_fee,
                price_impact,
                order_action,
                Some(market_data),
            );
        }
    }

    #[private]
    pub fn market_data_callback(
        &mut self,
        order_id: U128,
        mut order: Order,
        swap_fee: U128,
        price_impact: U128,
        order_action: OrderAction,
        market_data: Option<MarketData>,
    ) {
        let latest_market_data = if is_promise_success() {
            match env::promise_result(0) {
                PromiseResult::NotReady => MarketData::default(),
                PromiseResult::Successful(val) => {
                    if let Ok(data) = near_sdk::serde_json::from_slice::<MarketData>(&val) {
                        data
                    } else {
                        MarketData::default()
                    }
                }
                PromiseResult::Failed => MarketData::default(),
            }
        } else {
            market_data.unwrap()
        };

        if order_action == OrderAction::Cancel {
            self.final_order_cancel(order_id, order, latest_market_data, swap_fee, price_impact)
        } else {
            self.final_liquidate(order_id, order, latest_market_data);
        }
    }

    fn final_order_cancel(
        &mut self,
        order_id: U128,
        mut order: Order,
        latest_market_data: MarketData,
        swap_fee: U128,
        price_impact: U128,
    ) {
        let sell_amount =
            order.sell_token_price.value * BigDecimal::from(order.amount) * order.leverage;
        let pnl = self.calculate_pnl(signer_account_id(), order_id, latest_market_data);

        let expect_amount = self.get_price(order.buy_token.clone())
            * sell_amount
            * BigDecimal::from(10_u128.pow(24) - swap_fee.0)
            * BigDecimal::from(10_u128.pow(24) - price_impact.0)
            / order.buy_token_price.value;

        self.increase_balance(
            signer_account_id(),
            order.sell_token.clone(),
            expect_amount.round_u128() - order.amount * order.leverage.round_u128(),
        );

        if pnl.is_profit {
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
}
