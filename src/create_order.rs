use crate::*;

#[near_bindgen]
impl Contract {
    pub fn create_order(&mut self, order_type: String, amount: WBalance, sell_token: AccountId, buy_token: AccountId, leverage: U128) {}

    fn add_order(&mut self, account_id: AccountId, order: Order ) {}
}