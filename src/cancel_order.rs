use crate::*;

#[near_bindgen]
impl Contract {
    fn cancel_order(&mut self, _order_id: U128, _swap_fee: U128, _price_impact: U128) {}
}
