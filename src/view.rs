use crate::*;

#[near_bindgen]
impl Contract {
    pub fn view_market_data(&self, market: AccountId) -> MarketData {
        MarketData {
            total_supplies: U128(0),
            total_borrows: U128(0),
            total_reserves: U128(0),
            exchange_rate_ratio: U128(0),
            interest_rate_ratio: U128(0),
            borrow_rate_ratio: U128(0),
        }
    }

    pub fn view_order(&self, account_id: AccountId, order_id: U128) -> OrderView {
        OrderView {
            order_id,
            status: OrderStatus::Pending,
            order_type: OrderType::Buy,
            amount: 0,
            sell_token: "stoken".parse().unwrap(),
            buy_token: "btoken".parse().unwrap(),
            buy_token_price: U128(0),
            fee: U128(0),
        }
    }

    pub fn calculate_pnl(
        &self,
        account_id: AccountId,
        order_id: U128,
        data: MarketData,
    ) -> PnLView {
        PnLView {
            is_profit: true,
            amount: U128(0),
        }
    }

    pub fn view_orders(
        account_id: AccountId,
        sell_token: AccountId,
        buy_token: AccountId,
        market_borrow_apy: U128,
    ) -> Vec<OrderView> {
        let ov = OrderView {
            order_id: U128(0),
            status: OrderStatus::Pending,
            order_type: OrderType::Buy,
            amount: 0,
            sell_token,
            buy_token,
            buy_token_price: U128(0),
            fee: U128(0),
        };
        vec![ov]
    }
}
