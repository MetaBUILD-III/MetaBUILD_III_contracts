use crate::position::ViewPosition;
use crate::big_decimal::BigDecimal;
use crate::*;
use near_sdk::near_bindgen;
use std::str::FromStr;

#[near_bindgen]
impl Contract {
    pub fn view_balance(&self, user: AccountId, market: AccountId) -> WBalance {
        if self.user_profiles.get(&user).is_none() {
            U128::from(0)
        } else {
            let user_profile = self.user_profiles.get(&user).unwrap();

            if !user_profile.account_deposits.contains_key(&market) {
                U128::from(0)
            } else {
                U128::from(*user_profile.account_deposits.get(&market).unwrap())
            }
        }
    }

    #[allow(unused_variables)]
    pub fn view_user_positions(&self, market: AccountId, user: AccountId) -> Vec<ViewPosition> {
        if self.positions.get(&user).is_none() {
            return vec![];
        }

        self.positions
            .get(&user)
            .unwrap()
            .values()
            .map(|position| {
                let borrow_amount = U128::from(
                    BigDecimal::from(U128::from(position.collateral_amount))
                        * BigDecimal::from(U128::from(position.leverage)),
                );
                let price = self.get_price_by_token(position.buy_token.clone());

                ViewPosition {
                    active: position.active,
                    position_id: position.position_id.into(),
                    p_type: position.p_type.clone(),
                    amount: borrow_amount,
                    price,
                    fee: BigDecimal::from_str("0.3").unwrap().into(),
                    sell_token: position.sell_token.clone(),
                    buy_token: position.buy_token.clone(),
                }
            })
            .collect()
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

        let token_markets: Vec<(AccountId, AccountId)> = vec![
            (
                "usdt.qa.nearlend.testnet".parse().unwrap(),
                "usdt_market.qa.nearlend.testnet".parse().unwrap(),
            ),
            (
                "wnear.nearland.testnet".parse().unwrap(),
                "wnear_market.nearland.testnet".parse().unwrap(),
            ),
        ];

        let contract = Contract::new(token_markets);

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

        let token_markets: Vec<(AccountId, AccountId)> = vec![
            (
                "usdt.qa.nearlend.testnet".parse().unwrap(),
                "usdt_market.qa.nearlend.testnet".parse().unwrap(),
            ),
            (
                "wnear.nearland.testnet".parse().unwrap(),
                "wnear_market.nearland.testnet".parse().unwrap(),
            ),
        ];

        let contract = Contract::new(token_markets);

        let context = VMContextBuilder::new()
            .signer_account_id(owner_account.clone())
            .predecessor_account_id(owner_account)
            .build();

        testing_env!(context);

        dbg!(contract.view_user_positions(user_account, market));
    }
}
