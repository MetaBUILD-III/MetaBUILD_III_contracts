use crate::*;
use std::collections::HashMap;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Default)]
pub struct UserProfile {
    /// market address -> deposits
    pub account_deposits: HashMap<AccountId, Balance>,
}

impl UserProfile {
    pub fn new(market_id: AccountId, balance: Balance) -> UserProfile {
        let mut user_deposits = HashMap::new();
        user_deposits.insert(market_id, balance);
        UserProfile {
            account_deposits: user_deposits,
        }
    }
}
