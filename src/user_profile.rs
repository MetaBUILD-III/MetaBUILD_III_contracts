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
    pub fn new() -> UserProfile {
        UserProfile {
            account_deposits: Default::default(),
        }
    }
}
