use crate::ref_finance::ext_ref_finance;
use crate::utils::{ext_token, NO_DEPOSIT};
use crate::*;
use near_sdk::env::current_account_id;
use near_sdk::{ext_contract, is_promise_success, log, Gas, GasWeight, Promise};
use std::cmp::min;

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn remove_liquidity_for_execute_order_callback(&self, order: Order, order_id: U128);
}

#[near_bindgen]
impl Contract {
    pub fn execute_order(&self, order_id: U128) -> PromiseOrValue<U128> {
        let order = self.get_order_by(order_id.0);
        require!(order.is_some(), "There is no such order to be executed");

        let order = order.unwrap().clone();

        let remove_liquidity_amount = order.amount;

        let min_amount_x = 0;
        let min_amount_y = 0;
        // let min_amount_y = BigDecimal::from(order.amount- 1000) * (order.sell_token_price.value / order.buy_token_price.value);

        ext_ref_finance::ext(self.ref_finance_account.clone())
            .with_static_gas(Gas(10))
            .remove_liquidity(
                order.lpt_id.clone(),
                U128(remove_liquidity_amount),
                U128(min_amount_x),
                U128(min_amount_y),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(10))
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
            self.mark_order_as_executed(order.clone(), order_id);

            self.increase_balance(
                &env::signer_account_id(),
                &order.sell_token.clone(),
                reward_executor_amount,
            );

            let executor_reward_in_near = U128::from(env::used_gas().0 as u128 * 2u128);

            self.pay_to_executor(executor_reward_in_near, env::signer_account_id());

            return PromiseOrValue::Value(order_id);
        }
    }

    #[private]
    pub fn pay_to_executor(&self, amount: U128, to: AccountId) -> Promise {
        Promise::new(to).transfer(amount.0)
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
            &self.get_account_by(order_id.0).unwrap(), // assert there is always some user
            new_order.clone(),
            order_id.clone().0 as u64,
        );
    }

    pub fn get_account_by(&self, order_id: u128) -> Option<AccountId> {
        let mut account: Option<AccountId> = None;
        for (account_id, users_order) in self.orders.iter() {
            if users_order.contains_key(&(order_id as u64)) {
                account = Some(account_id);
                break;
            }
        }
        account
    }
}
