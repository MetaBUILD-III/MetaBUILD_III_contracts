use crate::*;
use near_sdk::ext_contract;
use near_sdk::serde::{Deserialize, Serialize};

pub type PoolId = String;
pub type LptId = String;

#[ext_contract(ext_ref_finance)]
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

    fn get_pool(&self, pool_id: PoolId);

    fn get_liquidity(&self, lpt_id: LptId);
}

/// Single swap action.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Swap {
    /// Pool which should be used for swapping.
    pub pool_ids: Vec<String>,
    /// Token to swap into.
    pub output_token: AccountId,
    /// Required minimum amount of token_out.
    pub min_output_amount: U128,
}

/// Single action. Allows to execute sequence of various actions initiated by an account.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
pub enum Action {
    SwapAction { Swap: Swap },
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct LiquidityInfo {
    pub lpt_id: LptId,
    pub owner_id: AccountId,
    pub pool_id: String,
    pub left_point: i32,
    pub right_point: i32,
    pub amount: U128,
    pub unclaimed_fee_x: U128,
    pub unclaimed_fee_y: U128,
}
