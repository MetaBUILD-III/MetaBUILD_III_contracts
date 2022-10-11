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

    pub fn view_order(&self, account_id: AccountId, order_id: U128) -> OrderView {}

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
}
