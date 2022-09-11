extern crate core;

mod common;
#[allow(unused_variables)]
mod user_profile;
mod utils;

const NO_DEPOSIT: u128 = 0;
const GAS_FOR_BORROW: Gas = Gas(180_000_000_000_000);
const WNEAR_MARKET: &str = "wnear_market.omomo-finance.testnet";

use crate::common::Events;
use crate::user_profile::UserProfile;
use crate::utils::WBalance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, UnorderedMap};
use near_sdk::env::current_account_id;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, is_promise_success, log, near_bindgen, require, AccountId, Balance,
    BorshStorageKey, Gas, PromiseOrValue, PromiseResult,
};

#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
enum PositionType {
    Long,
    Short,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct Position {
    active: bool,
    p_type: PositionType,
    sell_token: AccountId,
    buy_token: AccountId,
    collateral_amount: Balance,
    buy_token_price: Balance,
    sell_token_price: Balance,
    leverage: u128,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// Account of the owner.
    owner_id: AccountId,

    /// number of all positions
    total_positions: u128,

    /// list positions with data
    positions: LookupMap<u128, Position>,

    /// User Account ID -> market address -> collaterals
    /// User Account ID -> market address -> borrows
    user_profiles: UnorderedMap<AccountId, UserProfile>,
}

impl Default for Contract {
    fn default() -> Self {
        env::panic_str("Margin trading contract should be initialized before usage")
    }
}

#[derive(BorshSerialize, BorshStorageKey)]
pub enum StorageKeys {
    Positions,
    UserProfiles,
}

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn borrow_buy_token_callback(&self, amount: WBalance);
}

#[ext_contract(dtoken)]
trait MarketInterface {
    fn borrow(&mut self, amount: WBalance) -> PromiseOrValue<U128>;
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, _exchange_fee: u32, _referral_fee: u32) -> Self {
        require!(!env::state_exists(), "Already initialized");

        Self {
            owner_id,
            total_positions: 0,
            positions: LookupMap::new(StorageKeys::Positions),
            user_profiles: UnorderedMap::new(StorageKeys::UserProfiles),
        }
    }

    #[private]
    pub fn get_position(&self, position_id: U128) -> Position {
        self.positions
            .get(&position_id.0)
            .unwrap_or_else(|| panic!("Position with current position_id: {}", position_id.0))
    }

    pub fn open_position(
        &mut self,
        _amount: U128,
        _buy_token: AccountId,
        _sell_token: AccountId,
        _leverage: U128,
    ) -> u128 {
        self.total_positions += 1;
        self.total_positions
    }

    pub fn close_position(_position_id: U128) {}

    pub fn liquidate_position(_position_id: U128) {}

    pub fn borrow_buy_token(amount: U128) {
        require!(
            env::prepaid_gas() >= GAS_FOR_BORROW,
            "Prepaid gas is not enough for borrow flow"
        );

        assert!(
            Balance::from(amount) > 0,
            "Amount should be a positive number"
        );
        dtoken::ext(AccountId::try_from(WNEAR_MARKET.to_string()).unwrap())
            .with_static_gas(Gas(5))
            .with_attached_deposit(1)
            .borrow(amount)
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(3))
                    .with_attached_deposit(NO_DEPOSIT)
                    .borrow_buy_token_callback(amount),
            );
    }
}

impl Contract {
    fn borrow_buy_token_callback(&self, amount: Balance) {
        if !is_promise_success() {
            log!("{}", Events::BorrowFailedOnMarket(amount,));
        }

        // omomo market returns Balance of Borrow if so was successful
        let _borrow_balance = match env::promise_result(0) {
            PromiseResult::NotReady => 0,
            PromiseResult::Failed => 0,
            PromiseResult::Successful(result) => near_sdk::serde_json::from_slice::<U128>(&result)
                .unwrap()
                .into(),
        };

        // TODO make smth with borrow_balance further edit field of collateral
        // for some user that borrowed (could edit borrow_buy_token signature )
    }
}
