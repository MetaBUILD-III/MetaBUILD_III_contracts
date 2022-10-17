use near_sdk::env::signer_account_id;
use crate::*;
use crate::big_decimal::BigDecimal;

#[near_bindgen]
impl Contract {
    fn cancel_order(&mut self, order_id: U128, swap_fee: U128, price_impact: U128) {
        let orders = self.orders.get(&signer_account_id()).unwrap_or_else(||{
           panic!("Orders for account: {} not found", signer_account_id());
        });

        let order = orders.get(&(order_id.0 as u64)).unwrap_or_else(|| {
            panic!("Order with id: {} not found", order_id.0);
        });

        if order.status == OrderStatus::Pending {
            //remove liquadity
        }

        if order.leverage > BigDecimal::from(1) {

        }

        let market_data = self.market_infos.get().unwrap();
        let pnl = self.calculate_pnl(signer_account_id(), order_id, market_data);

        if pnl.is_profit {

        }

        self.update_balance() {

        }

        self.
    }
}
