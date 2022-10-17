use crate::*;

#[near_bindgen]
impl Contract {
    fn cancel_order(&mut self, order_id: U128, swap_fee: U128, price_impact: U128) {}
}
