use crate::*;
use near_sdk::FunctionError;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize, Debug)]
#[serde(crate = "near_sdk::serde")]
pub struct MarketData {
    pub total_supplies: WBalance,
    pub total_borrows: WBalance,
    pub total_reserves: WBalance,
    pub exchange_rate_ratio: WRatio,
    pub interest_rate_ratio: WRatio,
    pub borrow_rate_ratio: WRatio,
}

impl Default for MarketData {
    fn default() -> Self {
        Self {
            total_supplies: U128(0),
            total_borrows: U128(0),
            total_reserves: U128(0),
            exchange_rate_ratio: U128(0),
            interest_rate_ratio: U128(0),
            borrow_rate_ratio: U128(0),
        }
    }
}

#[near_bindgen]
impl Contract {
    pub fn update_market_data(&mut self, token_id: AccountId, market_id: AccountId) {
        underlying_token::ext(token_id.clone())
            .with_static_gas(Gas(12))
            .with_attached_deposit(1)
            .ft_balance_of(market_id.clone())
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(9))
                    .with_attached_deposit(NO_DEPOSIT)
                    .update_market_data_callback(token_id, market_id),
            );
    }

    pub fn update_market_data_callback(&self, token_id: AccountId, market_id: AccountId) {
        require!(is_promise_success(), "Market not have balance.");
        let market_balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Successful(val) => {
                near_sdk::serde_json::from_slice::<Balance>(&val).unwrap()
            }
            PromiseResult::Failed => 0,
        };

        require!(market_balance > 0, "Balance not fount.");
        ext_market::ext(utils::MARKET_PLATFORM_ACCOUNT.parse().unwrap())
            .with_static_gas(Gas(7))
            .with_attached_deposit(1)
            .view_market_data(U128(market_balance))
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(3))
                    .with_attached_deposit(NO_DEPOSIT)
                    .set_market_data(token_id, market_id),
            );
    }

    pub fn set_market_data(&mut self, token_id: AccountId, market_id: AccountId) {
        require!(is_promise_success(), "Some problem with market data.");
        let new_market_data = match env::promise_result(0) {
            PromiseResult::NotReady => MarketData::default(),
            PromiseResult::Successful(val) => {
                if let Ok(data) = near_sdk::serde_json::from_slice::<MarketData>(&val) {
                    data
                } else {
                    MarketData::default()
                }
            }
            PromiseResult::Failed => MarketData::default(),
        };
        let mut market_data = LookupMap::new(b"market_data".to_vec());
        market_data.insert(&market_id, &new_market_data);
        self.markets_data.insert(&token_id, &market_data);
    }

    pub fn get_market_data(&self, token_id: AccountId, market_id: AccountId) -> MarketData {
        let market = self.markets_data.get(&token_id).unwrap_or_else(|| {
            panic!("Market by token not found");
        });
        let market_data = market.get(&market_id).unwrap_or_else(|| {
            panic!("Market data by market id not found");
        });
        market_data
    }

    pub fn exchange_fee(&self, amount: Balance) -> Balance {
        let mut exchange_fee = 0;
        let trade_fee = self.trade_fee(amount);
        let admin_fee = (self.exchange_fee + self.referral_fee) as u128;
        let admin_fee_trade = self.admin_trade_fee(trade_fee, admin_fee);
        if self.referral_fee + self.exchange_fee > 0 {
            let mut fee_token = 0_u128;
            fee_token = admin_fee * self.referral_fee as u128
                / (self.referral_fee + self.exchange_fee) as u128;
            exchange_fee = admin_fee_trade - fee_token
        }
        exchange_fee
    }

    fn trade_fee(&self, amount: Balance) -> Balance {
        amount * self.total_fee as u128 / (utils::FEE_DIVISOR as u128)
    }

    fn admin_trade_fee(&self, amount: Balance, admin_fee: Balance) -> Balance {
        amount * admin_fee / (utils::FEE_DIVISOR as u128)
    }

    #[private]
    pub fn set_exchange_fee(&mut self, exchange_fee: U128) {
        self.exchange_fee = exchange_fee.0 as u32
    }

    pub fn get_exchange_fee(&self) -> U128 {
        U128(self.exchange_fee as u128)
    }

    #[private]
    pub fn set_referral_fee(&mut self, referral_fee: U128) {
        self.exchange_fee = referral_fee.0 as u32
    }

    pub fn get_referral_fee(&self) -> U128 {
        U128(self.referral_fee as u128)
    }

    #[private]
    pub fn set_total_fee(&mut self, total_fee: U128) {
        self.total_fee = total_fee.0 as u32
    }

    pub fn get_total_fee(&self) -> U128 {
        U128(self.total_fee as u128)
    }
}
