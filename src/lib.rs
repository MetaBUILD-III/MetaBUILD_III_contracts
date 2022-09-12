extern crate core;

mod common;
#[allow(unused_variables)]
mod user_profile;
mod utils;

const NO_DEPOSIT: u128 = 0;
const GAS_FOR_BORROW: Gas = Gas(180_000_000_000_000);
const WNEAR_MARKET: &str = "wnear_market.qa.nearlend.testnet";

use crate::common::Events;
use crate::user_profile::UserProfile;
use crate::utils::WBalance;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LookupSet, UnorderedMap};
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

    /// Market we are working with that are allowed to alter contracts field
    /// "wnear_market.omomo-finance.testnet", "usdt_market.omomo-finance.testnet"
    markets: LookupSet<AccountId>,
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
    Markets,
}

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn borrow_buy_token_callback(&self, amount: WBalance);
}

#[ext_contract(ext_market)]
trait MarketInterface {
    fn borrow(&mut self, amount: WBalance) -> PromiseOrValue<U128>;
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(markets: Vec<AccountId>) -> Self {
        require!(!env::state_exists(), "Already initialized");

        let mut lookup_markets = LookupSet::new(StorageKeys::Markets);
        for market in markets.iter() {
            lookup_markets.insert(market);
        }

        Self {
            owner_id,
            total_positions: 0,
            positions: LookupMap::new(StorageKeys::Positions),
            user_profiles: UnorderedMap::new(StorageKeys::UserProfiles),
            markets: lookup_markets,
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
        
        ext_market::ext(AccountId::try_from(WNEAR_MARKET.to_string()).unwrap())
            .with_static_gas(GAS_FOR_BORROW)
            .with_attached_deposit(NO_DEPOSIT)
            .borrow(amount)
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(3))
                    .with_attached_deposit(NO_DEPOSIT)
                    .borrow_buy_token_callback(amount),
            );
    }

    pub fn get_user_profile(&self, user_id: AccountId) -> UserProfile {
        self.user_profiles.get(&user_id).unwrap_or_default()
    }

    pub fn is_valid_market_call(&self) -> bool {
        self.markets.contains(&env::predecessor_account_id())
    }

    pub fn increase_user_deposit(
        &mut self,
        market_id: AccountId,
        user_id: AccountId,
        amount: WBalance,
    ) {
        assert!(
            self.is_valid_market_call(),
            "Only market is allowed to call this method"
        );

        // if its not present in our structure insert users profile
        if self.user_profiles.get(&user_id).is_none() {
            self.user_profiles.insert(&user_id, &UserProfile::new());
        }

        let mut user_profile: UserProfile = self.get_user_profile(user_id);

        // if user hasn't deposited yet
        if user_profile.account_deposits.get(&market_id).is_none() {
            user_profile
                .account_deposits
                .insert(market_id, Balance::from(amount));
        } else {
            user_profile.account_deposits.insert(
                market_id.clone(),
                user_profile.account_deposits.get(&market_id).unwrap() + Balance::from(amount),
            );
        }
    }

    pub fn decrease_user_deposit(
        &mut self,
        market_id: AccountId,
        user_id: AccountId,
        amount: WBalance,
    ) {
        assert!(
            self.is_valid_market_call(),
            "Only market is allowed to call this method"
        );

        assert!(self.user_profiles.get(&user_id).is_some());

        let mut user_profile: UserProfile = self.get_user_profile(user_id);

        // if user hasn't deposited yet
        if user_profile.account_deposits.get(&market_id).is_none() {
            user_profile
                .account_deposits
                .insert(market_id, Balance::from(amount));
        } else {
            let user_deposit_balance = user_profile.account_deposits.get(&market_id).unwrap();
            let decreased_user_deposit = user_deposit_balance - Balance::from(amount);
            assert!(
                decreased_user_deposit > 0,
                "Cannot be decreased to negative value"
            );
            user_profile
                .account_deposits
                .insert(market_id, decreased_user_deposit);
        }
    }

    #[private]
    pub fn borrow_buy_token_callback(&self, amount: U128) {
        if !is_promise_success() {
            log!("{}", Events::BorrowFailedOnMarket(amount.0));
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
