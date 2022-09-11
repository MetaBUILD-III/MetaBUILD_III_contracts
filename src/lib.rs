use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LookupSet};
use near_sdk::env::current_account_id;
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, is_promise_success, near_bindgen, require, AccountId, Balance, Gas,
    PromiseResult,
};

mod utils;

const NO_DEPOSIT: u128 = 0;

#[derive(Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
pub struct SwapAction {
    pub pool_id: u64,
    pub token_in: AccountId,
    pub amount_in: Option<U128>,
    pub token_out: AccountId,
    pub min_amount_out: U128,
}

#[derive(BorshSerialize, BorshDeserialize)]
enum PositionType {
    Long,
    Short,
}

#[ext_contract(ref_finance)]
pub trait RefFinanceInterface {
    /// tokens: pool tokens in this stable swap.
    /// decimals: each pool tokens decimal, needed to make them comparable.
    /// fee: total fee of the pool, admin fee is inclusive.
    /// amp_factor: algorithm parameter, decide how stable the pool will be.
    fn add_stable_swap_pool(
        &self,
        tokens: Vec<AccountId>,
        decimals: Vec<u8>,
        fee: u32,
        amp_factor: u64,
    ) -> u64;
    /// Execute set of swap actions between pools.
    /// If referrer provided, pays referral_fee to it.
    /// If no attached deposit, outgoing tokens used in swaps must be whitelisted.
    fn swap(&self, actions: Vec<SwapAction>, referral_id: Option<AccountId>) -> U128;
}

#[ext_contract(ext_self)]
pub trait ContractCallbackInterface {
    fn set_pool_id_callback(&mut self);
    fn swap_tokens_callback(&mut self);
}

#[derive(BorshDeserialize, BorshSerialize)]
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

    ///List of available tokens
    whitelisted_tokens: LookupSet<AccountId>,

    ///Pool id for swap
    pool_id: u64,
}

impl Default for Contract {
    fn default() -> Self {
        env::panic_str("Margin trading contract should be initialized before usage")
    }
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, _exchange_fee: u32, _referral_fee: u32) -> Self {
        require!(!env::state_exists(), "Already initialized");

        Self {
            owner_id,
            total_positions: 0,
            positions: LookupMap::new(b"positions".to_vec()),
            whitelisted_tokens: LookupSet::new(b"w_t".to_vec()),
            pool_id: 0,
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

    #[private]
    pub fn add_available_tokens(&mut self, tokens: Vec<AccountId>) {
        for token in tokens {
            self.whitelisted_tokens.insert(&token);
        }
    }

    #[private]
    pub fn remove_available_tokens(&mut self, tokens: Vec<AccountId>) {
        for token in tokens {
            self.whitelisted_tokens.remove(&token);
        }
    }

    #[private]
    pub fn set_pool_id(
        &self,
        tokens: Vec<AccountId>,
        decimals: Vec<u8>,
        fee: u32,
        amp_factor: u64,
    ) {
        ref_finance::ext(utils::get_ref_finance_account())
            .with_static_gas(Gas(5))
            .with_attached_deposit(1)
            .add_stable_swap_pool(tokens, decimals, fee, amp_factor)
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(3))
                    .with_attached_deposit(NO_DEPOSIT)
                    .set_pool_id_callback(),
            );
    }

    pub fn set_pool_id_callback(&mut self) {
        require!(is_promise_success(), "Pool was not created.");
        let pool_id = match env::promise_result(0) {
            PromiseResult::Successful(val) => {
                if let Ok(pool_id) = near_sdk::serde_json::from_slice::<u64>(&val) {
                    pool_id
                } else {
                    0
                }
            }
            PromiseResult::Failed => 0,
            _ => 0,
        };
        require!(pool_id > 0, "Pool was not created.");
        self.pool_id = pool_id;
    }

    pub fn swap_tokens(
        &mut self,
        token_in: AccountId,
        amount_in: Balance,
        token_out: AccountId,
        min_amount_out: Balance,
        referral_id: Option<AccountId>,
    ) {
        let mut actions: Vec<SwapAction> = Vec::new();
        let action = SwapAction {
            pool_id: self.pool_id,
            token_in,
            amount_in: Some(U128(amount_in)),
            token_out,
            min_amount_out: U128(min_amount_out),
        };
        actions.push(action);

        ref_finance::ext(utils::get_ref_finance_account())
            .with_static_gas(Gas(5))
            .with_attached_deposit(1)
            .swap(actions, referral_id)
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(3))
                    .with_attached_deposit(NO_DEPOSIT)
                    .swap_tokens_callback(),
            );
    }

    pub fn swap_tokens_callback(&mut self) {
        require!(is_promise_success(), "Pool was not created.");
        let _amount = match env::promise_result(0) {
            PromiseResult::Successful(val) => {
                if let Ok(amount) = near_sdk::serde_json::from_slice::<U128>(&val) {
                    amount
                } else {
                    U128(0)
                }
            }
            PromiseResult::Failed => U128(0),
            _ => U128(0),
        };
        //TODO: Do something with swap result
    }
}
