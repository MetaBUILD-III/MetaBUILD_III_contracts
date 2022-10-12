use crate::*;

#[near_bindgen]
impl Contract {
    pub fn view_market_data(&self, _market: AccountId) -> MarketData {
        MarketData {
            total_supplies: U128(0),
            total_borrows: U128(0),
            total_reserves: U128(0),
            exchange_rate_ratio: U128(0),
            interest_rate_ratio: U128(0),
            borrow_rate_ratio: U128(0),
        }
    }

    pub fn view_order(&self, _account_id: AccountId, order_id: U128) -> OrderView {
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
        _account_id: AccountId,
        _order_id: U128,
        _data: MarketData,
    ) -> PnLView {
        PnLView {
            is_profit: true,
            amount: U128(0),
        }
    }

    pub fn view_orders(
        _account_id: AccountId,
        sell_token: AccountId,
        buy_token: AccountId,
        _market_borrow_apy: U128,
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

    pub fn view_pair(&self, sell_token: AccountId, buy_token: AccountId) -> TradePair {
        self.supported_markets
            .get(&(sell_token, buy_token))
            .unwrap()
    }

    pub fn view_supported_pairs(&self) -> Vec<((AccountId, AccountId), TradePair)> {
        self.supported_markets.to_vec()
    }

    pub fn balance_of(&self, account_id: AccountId, token: AccountId) -> Balance {
        match self.balances.get(&account_id) {
            None => 0,
            Some(user_balance_per_token) => *user_balance_per_token.get(&token).unwrap_or(&0u128),
        }
    }
}
