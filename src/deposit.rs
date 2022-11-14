use crate::big_decimal::WBalance;
use crate::*;
use near_sdk::{Gas, PromiseOrValue};

const GAS_FOR_DEPOSIT: Gas = Gas(2_000_000_000_000);

impl Contract {
    pub fn deposit(
        &mut self,
        token_amount: WBalance,
        token: AccountId,
    ) -> PromiseOrValue<WBalance> {
        require!(
            env::prepaid_gas() >= GAS_FOR_DEPOSIT,
            "Prepaid gas is not enough for deposit flow"
        );

        let is_token_supported = self
            .supported_markets
            .keys()
            .any(|pair| pair.0 == token || pair.1 == token);

        require!(
            is_token_supported,
            "Deposit was done by token, that are not currently supported"
        );

        self.increase_balance(env::signer_account_id(), token, token_amount.0);

        PromiseOrValue::Value(U128(0))
    }

    pub fn increase_balance(
        &mut self,
        account_id: AccountId,
        token: AccountId,
        token_amount: Balance,
    ) {
        let increased_balance = self.balance_of(account_id.clone(), token.clone()) + token_amount;
        self.set_balance(account_id, token, increased_balance)
    }

    pub fn decrease_balance(
        &mut self,
        account_id: AccountId,
        token: AccountId,
        token_amount: Balance,
    ) {
        require!(
            self.balance_of(account_id.clone(), token.clone()) >= token_amount,
            "Decreased balance must be greater than 0"
        );
        self.set_balance(
            account_id.clone(),
            token.clone(),
            self.balance_of(account_id.clone(), token.clone()) - token_amount,
        )
    }

    pub fn set_balance(&mut self, account_id: AccountId, token: AccountId, token_amount: Balance) {
        let mut user_balance_by_token = self.balances.get(&account_id).unwrap_or_default();
        user_balance_by_token.insert(token, token_amount);
        self.balances.insert(&account_id, &user_balance_by_token);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    pub const INITIAL_BALANCE: Balance = 10_000;
    pub const AMOUNT_TO_INCREASE: Balance = 20_000;
    pub const AMOUNT_TO_DECREASE: Balance = 5_000;

    pub fn get_contract() -> (Contract, AccountId, AccountId) {
        let owner_id: AccountId = "contract.testnet".parse().unwrap();
        let oracle_account_id: AccountId = "oracle.testnet".parse().unwrap();

        let mut contract = Contract::new_with_config(owner_id, oracle_account_id);

        let user: AccountId = AccountId::from_str("some_example_user.testnet").unwrap();
        let token: AccountId = AccountId::from_str("some_example_token.testnet").unwrap();

        contract.set_balance(user.clone(), token.clone(), INITIAL_BALANCE);

        assert_eq!(
            contract.balance_of(user.clone(), token.clone()),
            INITIAL_BALANCE
        );

        (contract, user, token)
    }

    #[test]
    fn test_successful_increase_decrease_balance() {
        let (mut contract, user, token) = get_contract();

        contract.increase_balance(user.clone(), token.clone(), AMOUNT_TO_INCREASE);

        assert_eq!(
            contract.balance_of(user.clone(), token.clone()),
            AMOUNT_TO_INCREASE + INITIAL_BALANCE
        );

        contract.decrease_balance(user.clone(), token.clone(), AMOUNT_TO_DECREASE);

        assert_eq!(
            contract.balance_of(user.clone(), token.clone()),
            AMOUNT_TO_INCREASE + INITIAL_BALANCE - AMOUNT_TO_DECREASE
        );
    }

    #[test]
    #[should_panic]
    fn test_fail_decrease_balance() {
        let (mut contract, user, token) = get_contract();

        assert_eq!(
            contract.balance_of(user.clone(), token.clone()),
            INITIAL_BALANCE
        );

        contract.decrease_balance(user.clone(), token.clone(), 10000 * AMOUNT_TO_DECREASE);
    }
}
