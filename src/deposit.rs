use near_sdk::{Gas, PromiseOrValue};
use crate::*;
use crate::big_decimal::WBalance;


const GAS_FOR_DEPOSIT: Gas = Gas(120_000_000_000_000);


impl Contract {
    pub fn deposit(&mut self, token_amount: WBalance, token: AccountId) -> PromiseOrValue<WBalance> {
        require!(
            env::prepaid_gas() >= GAS_FOR_DEPOSIT,
            "Prepaid gas is not enough for deposit flow"
        );

        let is_token_supported = self.supported_markets.keys()
            .any(|pair| {
                pair.0 == token || pair.1 == token
            });


        assert!(is_token_supported, "Deposit was done by token, that are not currently supported");


        self.increase_balance(env::signer_account_id(), token, token_amount.0);

        PromiseOrValue::Value(U128(0))
    }


    pub fn increase_balance(&mut self, account_id: AccountId, token: AccountId, token_amount: Balance) {
        let increased_balance = self.balance_of(account_id.clone(), token.clone()) + token_amount;
        self.set_balance(account_id, token, increased_balance)
    }

    pub fn set_balance(&mut self, account_id: AccountId, token: AccountId, token_amount: Balance) {
        match self.balances.get(&account_id) {
            None => {
                let mut new_user_balance_by_token = HashMap::new();
                new_user_balance_by_token.insert(token, token_amount);

                self.balances.insert(&account_id, &new_user_balance_by_token);
            }
            Some(mut user_balance_by_token) => {
                user_balance_by_token.insert(token, token_amount);
                self.balances.insert(&account_id, &user_balance_by_token);
            }
        }
    }
}



