use crate::*;
use std::collections::HashMap;

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Default)]
pub struct UserProfile {
    /// Dtoken address -> collaterals
    pub account_supplies: HashMap<AccountId, Balance>,

    /// Dtoken address -> borrows
    pub account_borrows: HashMap<AccountId, Balance>,
}
