use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize, };
use near_sdk::collections::{LookupMap, LookupSet, Vector};
use near_sdk::json_types::U128;
use near_sdk::{env, near_bindgen, require, AccountId, Balance, BorshStorageKey};

mod utils;

#[derive(BorshStorageKey, BorshSerialize)]
enum StorageKey {
    Pools
}

#[derive(BorshSerialize, BorshDeserialize)]
enum PositionType {
    Long,
    Short,
}

fn swap(&mut self, actions: Vec<SwapAction>, referral_id: Option<AccountId>) -> U128 {

#[derive(BorshDeserialize, BorshSerialize)]
pub struct Position {
    active: bool,
    p_type: PositionType,
    sell_token: AccountId,
    buy_token: AccountId,
    collateral_amount:  Balance,
    buy_token_price: Balance,
    sell_token_price: Balance,
    leverage: u128
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
}

impl Default for Contract {
    fn default() -> Self {
        env::panic_str("Margin trading contract should be initialized before usage")
    }
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(owner_id: AccountId, exchange_fee: u32, referral_fee: u32) -> Self {
        require!(!env::state_exists(), "Already initialized");

        Self {
            owner_id: owner_id.clone(),
            total_positions: 0,
            positions: LookupMap::new(b"positions".to_vec()),
            whitelisted_tokens: LookupSet::new(b"w_t".to_vec()),
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
        amount: U128,
        buy_token: AccountId,
        sell_token: AccountId,
        leverage: U128,
    )->u128 {
        self.total_positions += 1;
        self.total_positions
    }

    pub fn close_position(position_id: U128) {}

    pub fn liquidate_position(position_id: U128) {}

    #[private]
    pub fn add_available_tokens(&mut self,  tokens: Vec<AccountId>) {
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

    pub fn swap_tokens(&mut self, tokens: Vec<AccountId>) {
        near call ref-exchange.testnet swap '{"actions": [{"pool_id": 100, "token_in": "nusdt.testnet", "amount_in": "1000000", "token_out": "nusdc.testnet", "min_amount_out": "990000"}], "referral_id": "referral.testnet"}' --account_id=user.testnet --amount=0.000000000000000001
    }
}
