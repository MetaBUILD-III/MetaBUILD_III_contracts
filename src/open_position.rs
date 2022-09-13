use crate::*;
use near_sdk::near_bindgen;

#[near_bindgen]
impl Contract {
    pub fn open_position(
        &mut self,
        sell_token: AccountId,
        sell_token_amount: U128,
        buy_token: AccountId,
        leverage: U128,
    ) -> PromiseOrValue<U128> {
        require!(self.user_profiles.get(&env::signer_account_id()).is_some(), "User have to deposit first");

        let user_profile = self.user_profiles.get(&env::signer_account_id()).unwrap();

        require!(user_profile.account_deposits.get(&sell_token).is_some(), "User don't have deposits in sell token");

        assert_eq!(*user_profile.account_deposits.get(&sell_token).unwrap(),
                   sell_token_amount.0,
                   "User don't have enough collateral deposited to proceed this action"
        );

        let xrate = self.calculate_xrate(sell_token.clone(), buy_token.clone());

        let borrow_token_amount =
            U128::from(Ratio::from(sell_token_amount) * xrate * Ratio::from(leverage));

        self.borrow_buy_token(borrow_token_amount);

        self.insert_position(env::signer_account_id(), Position::new(
            self.total_positions + 1,
            true,
            PositionType::Long,
            sell_token.clone(),
            buy_token.clone(),
            sell_token_amount.0,
            self.get_price_by_token(buy_token).0,
            self.get_price_by_token(sell_token).0,
            leverage.0,
        ));

        PromiseOrValue::Value(U128(0))
    }
}

impl Contract {
    pub fn insert_position(&mut self, user_id: AccountId, position: Position) {
        if self.positions.get(&user_id).is_none() {
            let mut position_by_id = HashMap::new();
            position_by_id.insert(*&position.position_id, position.clone());

            self.positions.insert(&user_id, &position_by_id);
        }

        self.positions.get(&user_id).unwrap().insert(*&position.position_id, position);

        self.total_positions += 1;
    }
}



