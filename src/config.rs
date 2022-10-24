use crate::*;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize, Clone)]
#[serde(crate = "near_sdk::serde")]
pub struct Config {
    /// The account ID of the contract owner that allows to modify config
    pub owner_id: AccountId,

    /// The account ID of the controller contract
    pub oracle_account_id: AccountId,
}

impl Contract {
    pub fn get_contract_config(&self) -> Config {
        self.config.clone()
    }
}
