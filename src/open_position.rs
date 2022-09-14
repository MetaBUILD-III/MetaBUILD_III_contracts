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
        require!(
            self.user_profiles.get(&env::signer_account_id()).is_some(),
            "User have to deposit first"
        );

        let user_profile = self.user_profiles.get(&env::signer_account_id()).unwrap();

        require!(
            user_profile.account_deposits.get(&sell_token).is_some(),
            "User don't have deposits in sell token"
        );

        require!(*user_profile.account_deposits.get(&sell_token).unwrap() >=
                   sell_token_amount.0,
                   "User don't have enough collateral deposited to proceed this action"
        );

        let xrate = self.calculate_xrate(sell_token.clone(), buy_token.clone());

        let borrow_token_amount = U128::from(
            Ratio::from(sell_token_amount) * Ratio::from(xrate) * Ratio::from(leverage),
        );
        log!("borrowing amount {}", borrow_token_amount.0);

        self.borrow_buy_token(borrow_token_amount.clone());

        self.insert_position(
            env::signer_account_id(),
            Position::new(
                self.total_positions,
                true,
                PositionType::Long,
                sell_token.clone(),
                buy_token.clone(),
                sell_token_amount.0,
                self.get_price_by_token(buy_token).0,
            self.get_price_by_token(sell_token).0,
                leverage.0,
                borrow_token_amount.0,
            ),
        );

        let borrow_amount = U128::from(
            Ratio::from(sell_token_amount) * Ratio::from(leverage),
        );
        PromiseOrValue::Value(borrow_amount)
    }
}

impl Contract {
    pub fn insert_position(&mut self, user_id: AccountId, position: Position) {
        self.decrease_user_deposit(position.sell_token.clone(), user_id.clone(), position.collateral_amount.into());

        let mut positions: HashMap<u128, Position> =if self.positions.get(&user_id).is_none() {
            HashMap::new()
        } else {
            self.positions
            .get(&user_id)
            .unwrap()
        };

        positions.insert(*&position.position_id, position);
        self.positions.insert(&user_id, &positions);
            
        self.total_positions += 1;
    }
}
