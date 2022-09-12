use std::borrow::Borrow;
use crate::position::ViewPosition;
use crate::ratio::Ratio;
use crate::*;
use near_sdk::near_bindgen;
use std::str::FromStr;

const USDT_MARKET: &str = "usdt.qa.nearlend.testnet";

#[near_bindgen]
impl Contract {
    pub fn view_balance(&self, user: AccountId, market: AccountId) -> WBalance {
        if self.user_profiles.get(&user).is_none() {
            U128::from(0)
        } else {
            U128::from(
                *self
                    .user_profiles
                    .get(&user)
                    .unwrap()
                    .account_deposits
                    .get(&AccountId::from_str(USDT_MARKET).unwrap())
                    .unwrap_or(&(U128::from(Ratio::from_str("321.432").unwrap()).0 * 10u128.pow(24)))
                // TODO change to .unwrap_or(&0),
            )
        }
    }

    pub fn view_user_positions(&self, market: AccountId, user: AccountId) -> Vec<ViewPosition> {
        return vec![
            ViewPosition::new(
                10000000000000000000000000,
                1000 * 10u128.pow(24),
                Ratio::from_str("0.3").unwrap(),
            ),
            ViewPosition::new(
                48729500000000000000000000,
                20000 * 10u128.pow(24),
                Ratio::from_str("0.3").unwrap(),
            ),
            ViewPosition::new(
                646382000000000000000000000,
                4400 * 10u128.pow(24),
                Ratio::from_str("0.15").unwrap(),
            ),
            ViewPosition::new(
                30000000000000000000000000,
                24230 * 10u128.pow(24),
                Ratio::from_str("0.11").unwrap(),
            ),
        ];
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use near_sdk::test_utils::test_env::{alice, bob};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::testing_env;
    use std::str::FromStr;

    #[test]
    fn test_view_balance() {
        let (owner_account, user_account, market) = (
            alice(),
            bob(),
            AccountId::from_str("usdt.qa.nearlend.testnet").unwrap(),
        );

        let contract = Contract::new(vec![
            AccountId::from_str("usdt.qa.nearlend.testnet").unwrap()
        ]);

        let context = VMContextBuilder::new()
            .signer_account_id(owner_account.clone())
            .predecessor_account_id(owner_account)
            .build();

        testing_env!(context);

        assert_eq!(contract.view_balance(user_account, market), U128(0));
    }

    #[test]
    fn test_view_user_positions() {
        let (owner_account, user_account, market) = (
            alice(),
            bob(),
            AccountId::from_str("usdt.qa.nearlend.testnet").unwrap(),
        );

        let contract = Contract::new(vec![
            AccountId::from_str("usdt.qa.nearlend.testnet").unwrap()
        ]);

        let context = VMContextBuilder::new()
            .signer_account_id(owner_account.clone())
            .predecessor_account_id(owner_account)
            .build();

        testing_env!(context);

        dbg!(contract.view_user_positions(user_account, market));
    }
}
