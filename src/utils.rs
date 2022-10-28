use crate::*;
use near_sdk::ext_contract;

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
    pub fn get_order_by(&self, order_id: u128) -> Option<Order> {
        let account = self.get_account_by(order_id).unwrap();

        self.orders
            .get(&account)
            .unwrap()
            .get(&(order_id as u64))
            .cloned()
    }
}

#[ext_contract(ext_market)]
pub trait MarketInterface {
    fn borrow(&mut self, amount: WBalance) -> PromiseOrValue<U128>;
    fn view_market_data(&self) -> MarketData;
}
