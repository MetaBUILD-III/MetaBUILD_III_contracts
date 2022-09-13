use crate::utils::ext_token;
use crate::BigDecimal;
use crate::{Contract, ContractExt};

use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{near_bindgen, AccountId, Balance, Gas, PromiseOrValue};

pub const REF_FINANCE: &str = "ref-finance-101.testnet";

/// Single swap action.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapAction {
    /// Pool which should be used for swapping.
    pub pool_id: u64,
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

/// Message parameters to receive via token function call.
#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[serde(untagged)]
enum TokenReceiverMessage {
    /// Alternative to deposit + execute actions call.
    Execute {
        force: bool,
        /// List of sequential actions.
        actions: Vec<Action>,
    },
}

#[near_bindgen]
impl Contract {
    pub fn close_position(&mut self, position_id: U128) -> PromiseOrValue<Balance> {
        let position = self.get_position(position_id);
        // TODO check for position owner

        // TODO Receive min_amount_out (from UI?)
        let min_amount_out = U128(1u128);
        // TODO Better calculus
        let borrowed_amount =
            BigDecimal::from(position.collateral_amount) * BigDecimal::from(position.leverage);
        self.execute_position(
            position.sell_token,
            borrowed_amount.into(),
            position.buy_token,
            min_amount_out,
        )
    }

    // TODO hide from near bindgen
    pub fn execute_position(
        &mut self,
        token_in: AccountId,
        amount_in: U128,
        token_out: AccountId,
        min_amount_out: U128,
    ) -> PromiseOrValue<Balance> {
        let actions: Vec<Action> = vec![Action::Swap(SwapAction {
            pool_id: 1734,
            token_in: token_in.clone(),
            amount_in: Some(amount_in),
            token_out,
            min_amount_out,
        })];

        let action = TokenReceiverMessage::Execute {
            force: false,
            actions,
        };

        ext_token::ext(token_in)
            .with_static_gas(Gas(3))
            .with_attached_deposit(1)
            .ft_transfer_call(
                REF_FINANCE.parse().unwrap(),
                amount_in,
                Some("Deposit tokens".to_string()),
                near_sdk::serde_json::to_string(&action).unwrap(),
            )
            .into()
        // TODO handle successful swap
    }
}
