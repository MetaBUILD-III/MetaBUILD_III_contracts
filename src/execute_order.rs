use crate::ref_finance::{ref_finance, Action, SwapAction, TokenReceiverMessage};
use crate::utils::{ext_market, ext_token, NO_DEPOSIT};
use crate::*;
use near_sdk::env::current_account_id;
use near_sdk::{ext_contract, is_promise_success, Gas};
use std::collections::VecDeque;

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn remove_liquidity_for_execute_order_callback(&self, order: Order, order_id: U128);
}

#[near_bindgen]
impl Contract {
    pub fn execute_order(&self, order_id: U128) -> PromiseOrValue<U128> {
        assert!(
            self.get_order_by_id(order_id.0 as u64).is_some(),
            "There is no such order to be executed"
        );

        let order = self.get_order_by_id(order_id.0 as u64).unwrap().clone();

        // TODO set real arguments
        let amount = 1;
        let min_amount_x = order.amount;
        let min_amount_y = 0;

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
                    .remove_liquidity_for_execute_order_callback(order, order_id),
            )
            .into()
    }

    #[private]
    pub fn remove_liquidity_for_execute_order_callback(
        &mut self,
        order: Order,
        order_id: U128,
    ) -> PromiseOrValue<U128> {
        if !is_promise_success() {
            panic!("Some problem with remove liquidity");
        } else {
            self.mark_order_as_executed(order, order_id);

            let reward_executor_amount = order.amount.clone() * 10u128.pow(23); // reward is 0.1% from sell_token_amount

            self.increase_balance(
                env::signer_account_id(),
                order.sell_token.clone(),
                reward_executor_amount,
            );

            ext_token::ext(order.sell_token.clone())
                .with_static_gas(Gas(10))
                .with_attached_deposit(1)
                .ft_transfer(env::signer_account_id(), U128::from(reward_executor_amount));

            return PromiseOrValue::Value(order_id);
        }
    }
}

impl Contract {
    pub fn mark_order_as_executed(&mut self, order: Order, order_id: U128) {
        let order = order.clone();

        let new_order = Order {
            status: OrderStatus::Executed,
            order_type: order.order_type,
            amount: order.amount,
            sell_token: order.sell_token,
            buy_token: order.buy_token,
            leverage: order.leverage,
            sell_token_price: order.sell_token_price,
            buy_token_price: order.buy_token_price,
            block: order.block,
            lpt_id: order.lpt_id,
        };

        self.insert_order_for_user(
            &self.get_user_by_order_id(order_id.0).unwrap(), // assert there is always some user
            new_order.clone(),
            order_id.clone().0 as u64,
        );
    }

    pub fn get_user_by_order_id(&self, order_id: u128) -> Option<AccountId> {
        let mut outcome = VecDeque::new();

        for (account_id, users_order) in self.orders.iter() {
            match users_order.contains_key(&(order_id as u64)) {
                true => outcome.push_front(account_id),
                false => (),
            }
        }

        outcome.pop_front()
    }
}
