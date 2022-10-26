use crate::big_decimal::{BigDecimal, WRatio};
use crate::ref_finance::ref_finance;
use crate::ref_finance::{Action, SwapAction, TokenReceiverMessage};
use crate::utils::NO_DEPOSIT;
use crate::utils::{ext_market, ext_token};
use crate::*;
use near_sdk::env::{current_account_id, signer_account_id};
use near_sdk::{ext_contract, is_promise_success, log, Gas, PromiseResult};

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn remove_liquidity_callback(&self, order_id: U128, swap_fee: U128, price_impact: U128);
    fn order_cancel_swap_callback(&mut self, order_id: U128, swap_fee: U128, price_impact: U128);
    fn market_data_callback(
        &mut self,
        order_id: U128,
        swap_fee: U128,
        price_impact: U128,
        market_data: Option<MarketData>,
    );
}

#[near_bindgen]
impl Contract {
    fn cancel_order(&mut self, order_id: U128, swap_fee: U128, price_impact: U128) {
        let orders = self.orders.get(&signer_account_id()).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", signer_account_id());
        });

        let order = orders.get(&(order_id.0 as u64)).unwrap_or_else(|| {
            panic!("Order with id: {} not found", order_id.0);
        });

        //TODO: set real min_amount_x/min_amount_y
        let amount = 1;
        let min_amount_x = order.amount;
        let min_amount_y = 0;

        if order.status == OrderStatus::Pending {
            ref_finance::ext(self.ref_finance_account.clone())
                .with_static_gas(Gas(10))
                .with_attached_deposit(1)
                .remove_liquidity(
                    order.lpt_id.clone(),
                    U128(amount),
                    U128(min_amount_x),
                    U128(min_amount_y),
                )
                .then(
                    ext_self::ext(current_account_id())
                        .with_static_gas(Gas(5))
                        .with_attached_deposit(NO_DEPOSIT)
                        .remove_liquidity_callback(order_id, swap_fee, price_impact),
                );
        } else {
            self.swap(order_id, swap_fee, price_impact);
        }
    }

    #[private]
    pub fn remove_liquidity_callback(
        &mut self,
        order_id: U128,
        swap_fee: U128,
        price_impact: U128,
    ) {
        require!(is_promise_success(), "Some problem with remove liquidity");
        self.order_cancel_swap_callback(order_id, swap_fee, price_impact);
    }

    fn swap(&self, order_id: U128, swap_fee: U128, price_impact: U128) {
        let orders = self.orders.get(&signer_account_id()).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", signer_account_id());
        });

        let order = orders.get(&(order_id.0 as u64)).unwrap_or_else(|| {
            panic!("Order with id: {} not found", order_id.0);
        });

        let buy_amount =
            BigDecimal::from(order.amount) * order.leverage * order.sell_token_price.value
                / order.buy_token_price.value;
        let min_amount = buy_amount * self.get_price(order.buy_token.clone());
        let actions: Vec<Action> = vec![Action::Swap(SwapAction {
            pool_id: self.pool_id,
            token_in: order.buy_token.clone(),
            amount_in: Some(WRatio::from(buy_amount)),
            token_out: order.sell_token.clone(),
            min_amount_out: WRatio::from(min_amount),
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
                Some("Deposit tokens".to_string()),
                near_sdk::serde_json::to_string(&action).unwrap(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(20))
                    .with_attached_deposit(NO_DEPOSIT)
                    .order_cancel_swap_callback(order_id, swap_fee, price_impact),
            );
    }

    #[private]
    pub fn order_cancel_swap_callback(
        &mut self,
        order_id: U128,
        swap_fee: U128,
        price_impact: U128,
    ) {
        require!(is_promise_success(), "Some problem tish swap tokens");

        let orders = self.orders.get(&signer_account_id()).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", signer_account_id());
        });

        let order = orders.get(&(order_id.0 as u64)).unwrap_or_else(|| {
            panic!("Order with id: {} not found", order_id.0);
        });

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
                        .market_data_callback(order_id, swap_fee, price_impact, None),
                );
        } else {
            let market_data = self.market_infos.get(&market_id).unwrap();
            self.market_data_callback(order_id, swap_fee, price_impact, Some(market_data));
        }
    }

    #[private]
    pub fn market_data_callback(
        &mut self,
        order_id: U128,
        swap_fee: U128,
        price_impact: U128,
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

        let orders = self.orders.get(&signer_account_id()).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", signer_account_id());
        });

        let order = orders.get(&(order_id.0 as u64)).unwrap_or_else(|| {
            panic!("Order with id: {} not found", order_id.0);
        });

        let sell_amount =
            order.sell_token_price.value * BigDecimal::from(order.amount) * order.leverage;
        let pnl = self.calculate_pnl(signer_account_id(), order_id, latest_market_data);

        let expect_amount = self.get_price(order.buy_token.clone())
            * sell_amount
            * BigDecimal::from(10_u128.pow(24) - swap_fee.0)
            * BigDecimal::from(10_u128.pow(24) - price_impact.0)
            / order.buy_token_price.value;

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
    }
}
