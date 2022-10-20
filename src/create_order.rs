use std::ops::Range;
use near_sdk::{Gas, log};
use crate::*;
use crate::big_decimal::{BigDecimal, WBalance};


const NO_DEPOSIT: u128 = 0;
const GAS_FOR_BORROW: Gas = Gas(180_000_000_000_000);

#[near_bindgen]
impl Contract {
    pub fn create_order(&mut self, order_type: String, amount: WBalance, sell_token: AccountId, buy_token: AccountId, leverage: U128) {
        let user = env::signer_account_id();

        // TODO more complicated assertion for balance
        let user_balance = self.balance_of(user.clone(), sell_token.clone());
        // self.set_balance(user, sell_token, user_balance - amount);

        let xrate = self.calculate_xrate(sell_token.clone(), buy_token.clone());

        if BigDecimal::from(leverage) > BigDecimal::one() {
            let borrow_token_amount =
                U128::from(BigDecimal::from(amount) * xrate * BigDecimal::from(leverage));
            log!("borrowing amount {}", borrow_token_amount.0);

            self.borrow_buy_token(borrow_token_amount, buy_token);
        } else {

        }
    }

    #[private]
    pub fn add_order(&mut self, account_id: AccountId, order: Order) {}


    pub fn borrow_buy_token(&self, amount: U128, market: AccountId) {
        require!(
            env::prepaid_gas() >= GAS_FOR_BORROW,
            "Prepaid gas is not enough for borrow flow"
        );

        assert!(
            Balance::from(amount) > 0,
            "Amount should be a positive number"
        );


        ext_market::ext(AccountId::try_from(WNEAR_MARKET.to_string()).unwrap())
            .with_static_gas(GAS_FOR_BORROW)
            .with_attached_deposit(NO_DEPOSIT)
            .borrow(amount);
    }
}