use crate::*;
use near_sdk::ext_contract;
use std::borrow::Borrow;
use std::collections::VecDeque;
use std::hash::Hash;

pub const NO_DEPOSIT: Balance = 0;

#[ext_contract(ext_token)]
pub trait NEP141Token {
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: WBalance,
        memo: Option<String>,
        msg: String,
    );

    fn ft_transfer(&mut self, receiver_id: AccountId, amount: WBalance, memo: Option<String>);
}

impl Contract {
    pub fn get_order_by_id(&self, order_id: u64) -> Option<Order> {
        let mut outcome = self
            .orders
            .values()
            .filter(|hashmap_by_order| hashmap_by_order.contains_key(&order_id))
            .map(|mut correct_one| correct_one.get(&order_id).unwrap().clone())
            .collect::<VecDeque<Order>>();

        outcome.pop_front()
    }
}

#[ext_contract(ext_market)]
pub trait MarketInterface {
    fn borrow(&mut self, amount: WBalance) -> PromiseOrValue<U128>;
    fn view_market_data(&self) -> MarketData;
}
