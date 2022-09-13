use crate::utils::*;
use crate::{BigDecimal, Position, Ratio, WRatio, NO_DEPOSIT};
use crate::{Contract, ContractExt};

use near_sdk::env::{current_account_id, signer_account_id};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    ext_contract, is_promise_success, near_bindgen, require, AccountId, Balance, Gas,
    PromiseOrValue,
};

pub const REF_FINANCE: &str = "ref-finance-101.testnet";
#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn swap_callback(&mut self, amount: Balance, position_id: U128) -> PromiseOrValue<Balance>;
}

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
    /// calculate all fees
    /// execute swap of sell token to buy token
    /// deduce all fees from the resulting amount of buy token
    /// deduce profit fee = 10% (if position is profitable)
    pub fn close_position(&mut self, position_id: U128) -> PromiseOrValue<Balance> {
        let position = self.get_position(position_id);
        require!(
            signer_account_id() == position.owner,
            "Signer not position owner"
        );
        require!(position.active, "Position not active.");

        // TODO Receive min_amount_out (from UI?)
        let min_amount_out = U128(1u128);

        self.execute_position(position_id, min_amount_out).into()
    }

    fn execute_position(
        &mut self,
        position_id: U128,
        min_amount_out: U128,
    ) -> PromiseOrValue<Balance> {
        let position = self.get_position(position_id);
        let amount_in = WRatio::from(
            BigDecimal::from(position.collateral_amount) * BigDecimal::from(position.leverage),
        );
        let actions: Vec<Action> = vec![Action::Swap(SwapAction {
            pool_id: self.pool_id,
            token_in: position.buy_token.clone(),
            amount_in: Some(amount_in),
            token_out: position.sell_token.clone(),
            min_amount_out,
        })];

        let action = TokenReceiverMessage::Execute {
            force: false,
            actions,
        };

        ext_token::ext(position.buy_token.clone())
            .with_static_gas(Gas(3))
            .with_attached_deposit(1)
            .ft_transfer_call(
                REF_FINANCE.parse().unwrap(),
                amount_in,
                Some("Deposit tokens".to_string()),
                near_sdk::serde_json::to_string(&action).unwrap(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(20))
                    .with_attached_deposit(NO_DEPOSIT)
                    .swap_callback(amount_in.0, position_id),
            )
            .into()
    }

    #[private]
    pub fn swap_callback(&mut self, amount: Balance, position_id: U128) -> PromiseOrValue<Balance> {
        require!(is_promise_success(), "Some problem with swap tokens");
        let mut position = self.get_position(position_id);
        let market_id = self.tokens_markets.get(&position.sell_token).unwrap();
        let market_data = self.get_market_data(position.sell_token.clone(), market_id.clone());
        let borrow_fee = market_data.borrow_rate_ratio.0;
        let swap_fee = self.exchange_fee(amount);
        let fee = borrow_fee + swap_fee;

        self.decrease_user_deposit(market_id.clone(), signer_account_id(), U128(fee));

        let sell_token_price = self.get_price_by_token(position.sell_token.clone());
        let pnl = self.calculate_pnl(
            U128(position.buy_token_price),
            sell_token_price,
            U128(position.collateral_amount),
            U128(position.leverage),
        );
        if pnl.0 {
            let fee_amount = Ratio::from(position.collateral_amount) / Ratio::from(10_u128);
            self.decrease_user_deposit(
                market_id.clone(),
                signer_account_id(),
                WRatio::from(fee_amount),
            );
            self.increase_user_deposit(market_id, signer_account_id(), WRatio::from(pnl.1));
        } else {
            self.decrease_user_deposit(market_id, signer_account_id(), WRatio::from(pnl.1));
        }
        position.active = false;
        self.positions.insert(&position_id.0, &position);
        PromiseOrValue::Value(amount)
    }

    #[private]
    pub fn set_pool_id(&mut self, pool_id: U128) {
        self.pool_id = pool_id.0 as u64;
    }
}
