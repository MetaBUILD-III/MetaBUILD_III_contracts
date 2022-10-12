use crate::common::Actions;
use crate::*;
use near_contract_standards::fungible_token::receiver::FungibleTokenReceiver;
use near_sdk::json_types::U128;
use near_sdk::AccountId;
use near_sdk::{log, serde_json, Balance, PromiseOrValue};

#[near_bindgen]
impl FungibleTokenReceiver for Contract {
    fn ft_on_transfer(
        &mut self,
        sender_id: AccountId,
        amount: U128,
        msg: String,
    ) -> PromiseOrValue<U128> {
        // assert_eq!(
        //     env::predecessor_account_id(),
        //     self.underlying_token,
        //     "The call should come from token account"
        // );

        assert!(
            Balance::from(amount) > 0,
            "Amount should be a positive number"
        );

        log!(format!("sender_id {}, msg {}", sender_id, msg));

        let action: Actions = serde_json::from_str(&msg).expect("Incorrect command in transfer");

        match action {
            Actions::Deposit { token } => self.deposit(amount, token),
            _ => {
                panic!("Incorrect action in transfer")
            }
        }
    }
}
