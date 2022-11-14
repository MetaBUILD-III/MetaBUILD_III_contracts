use crate::*;
use near_sdk::ext_contract;
use near_sdk::serde::{Deserialize, Serialize};

pub type LptId = String;

#[ext_contract(ref_finance)]
trait RefFinanceInterface {
    fn add_liquidity(
        &mut self,
        pool_id: String,
        left_point: i32,
        right_point: i32,
        amount_x: U128,
        amount_y: U128,
        min_amount_x: U128,
        min_amount_y: U128,
    );

    fn remove_liquidity(
        &self,
        lpt_id: LptId,
        amount: U128,
        min_amount_x: U128,
        min_amount_y: U128,
    ) -> (U128, U128);

    fn get_pool(&self, pool_id: LptId);
}

/// Message parameters to receive via token function call.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
pub enum TokenReceiverMessage {
    /// Alternative to deposit + execute actions call.
    Execute {
        force: bool,
        /// List of sequential actions.
        actions: Vec<Action>,
    },
}

/// Single swap action.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapAction {
    /// Pool which should be used for swapping.
    pub pool_id: String,
    /// Token to swap from.
    pub token_in: AccountId,
    /// Amount to exchange.
    /// If amount_in is None, it will take amount_out from previous step.
    /// Will fail if amount_in is None on the first step.
    pub amount_in: Option<U128>,
    /// Token to swap into.
    pub token_out: AccountId,
    /// Required minimum amount of token_out.
    pub min_amount_out: U128,
}

/// Single action. Allows to execute sequence of various actions initiated by an account.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
pub enum Action {
    Swap(SwapAction),
}
