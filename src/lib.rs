extern crate core;
mod close_position;
mod common;
mod fee;
mod open_position;
mod position;
mod price;
mod big_decimal;
mod user_profile;
mod utils;
mod views;

const NO_DEPOSIT: u128 = 0;
const GAS_FOR_BORROW: Gas = Gas(180_000_000_000_000);
const WNEAR_MARKET: &str = "wnear_market.qa.nearlend.testnet";

use crate::common::Events;
use crate::fee::MarketData;
use crate::big_decimal::*;
use crate::user_profile::UserProfile;
use crate::utils::{ext_token, WBalance};

use std::collections::HashMap;
use std::hash::Hash;
use std::str::FromStr;

use crate::price::Price;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LookupMap, LookupSet, UnorderedMap};
use near_sdk::env::{current_account_id, signer_account_id};
use near_sdk::json_types::U128;
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{
    env, ext_contract, is_promise_success, log, near_bindgen, require, AccountId, Balance,
    BorshStorageKey, Gas, PromiseOrValue, PromiseResult,
};


#[derive(BorshSerialize, BorshDeserialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug, Clone)]
pub enum PositionType {
    Long,
    Short,
}

#[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
#[serde(crate = "near_sdk::serde")]
#[derive(Debug, Clone)]
pub struct Position {
    position_id: u128,
    active: bool,
    p_type: PositionType,
    sell_token: AccountId,
    buy_token: AccountId,
    collateral_amount: Balance,
    buy_token_price: Balance,
    sell_token_price: Balance,
    leverage: u128,
    borrow_amount: Balance,
}

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize)]
pub struct Contract {
    /// number of all positions
    total_positions: u128,

    ///  user_id -> position_id -> position
    positions: UnorderedMap<AccountId, HashMap<u128, Position>>,

    /// User Account ID -> market address -> collaterals
    /// User Account ID -> market address -> borrows
    user_profiles: UnorderedMap<AccountId, UserProfile>,

    /// Market we are working with that are allowed to alter contracts field
    /// "wnear_market.qa.nearlend.testnet", "usdt_market.qa.nearlend.testnet"
    markets: LookupSet<AccountId>,

    /// token ID -> Price
    pub prices: LookupMap<AccountId, Price>,

    /// token id -> market id
    tokens_markets: LookupMap<AccountId, AccountId>,

    /// pool fee
    total_fee: u32,

    /// Exchange fee, that goes to exchange itself (managed by governance).
    exchange_fee: u32,

    /// Referral fee, that goes to referrer in the call.
    referral_fee: u32,

    /// List of all the pools.
    /// data about markets
    /// first AccountId -> Token
    /// second AccountId -> Market
    markets_data: LookupMap<AccountId, LookupMap<AccountId, MarketData>>,

    /// Pool which should be used for swapping.
    pool_id: u64,
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
    Prices,
    MarketsData,
    TokenMarkets,
    Vector,
}

#[ext_contract(underlying_token)]
trait UnderlineTokenInterface {
    fn ft_balance_of(&self, account_id: AccountId) -> U128;
}

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn borrow_buy_token_callback(&self, amount: WBalance);
    fn update_market_data_callback(&self, token_id: AccountId, market_id: AccountId);
    fn set_market_data(&self, token_id: AccountId, market_id: AccountId);
    fn withdraw_callback(&self, token_id: AccountId, amount: U128);
}

#[ext_contract(ext_market)]
trait MarketInterface {
    fn borrow(&mut self, amount: WBalance) -> PromiseOrValue<U128>;
    fn view_market_data(&self, ft_balance: WBalance) -> MarketData;
}

#[near_bindgen]
impl Contract {
    #[init]
    pub fn new(tokens_markets: Vec<(AccountId, AccountId)>) -> Self {
        require!(!env::state_exists(), "Already initialized");

        let mut lookup_markets = LookupSet::new(StorageKeys::Markets);
        let mut lookup_tm = LookupMap::new(StorageKeys::TokenMarkets);
        for tm in tokens_markets.iter() {
            lookup_tm.insert(&tm.0, &tm.1);
            lookup_markets.insert(&tm.1);
        }

        Self {
            total_positions: 0,
            positions: UnorderedMap::new(StorageKeys::Positions),
            user_profiles: UnorderedMap::new(StorageKeys::UserProfiles),
            markets: lookup_markets,
            prices: LookupMap::new(StorageKeys::Prices),
            tokens_markets: lookup_tm,
            total_fee: 0,
            exchange_fee: 0,
            referral_fee: 0,
            markets_data: LookupMap::new(StorageKeys::MarketsData),
            pool_id: 0,
        }
    }

    #[private]
    pub fn get_position(&self, position_id: U128) -> Position {
        self.positions
            .get(&env::signer_account_id())
            .unwrap()
            .get(&position_id.0)
            .unwrap()
            .clone()
    }

    pub fn liquidate_position(_position_id: U128) {}

    pub fn borrow_buy_token(&self, amount: U128) {
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

    pub fn increase_user_deposit(
        &mut self,
        market_id: AccountId,
        user_id: AccountId,
        amount: WBalance,
    ) {
        // assert!(
        //     self.is_valid_market_call(),
        //     "Only market is allowed to call this method"
        // );

        // if its not present in our structure insert users profile
        if self.user_profiles.get(&user_id).is_none() {
            self.user_profiles
                .insert(&user_id, &UserProfile::new(market_id.clone(), 0u128));
        }

        let mut user_profile: UserProfile = self.get_user_profile(user_id.clone());

        // if user has UserProfile, but deposited in different token
        if user_profile.account_deposits.get(&market_id).is_none() {
            user_profile.account_deposits.insert(market_id, amount.0);
            self.user_profiles.insert(&user_id, &user_profile);
        } else {
            let increased_balance =
                amount.0 + *user_profile.account_deposits.get(&market_id).unwrap();
            user_profile
                .account_deposits
                .insert(market_id, increased_balance);
        }
        self.user_profiles.insert(&user_id, &user_profile);
    }

    pub fn decrease_user_deposit(
        &mut self,
        market_id: AccountId,
        user_id: AccountId,
        amount: WBalance,
    ) {
        // assert!(
        //     self.is_valid_market_call(),
        //     "Only market is allowed to call this method"
        // );

        assert!(self.user_profiles.get(&user_id).is_some());

        let mut user_profile: UserProfile = self.get_user_profile(user_id.clone());

        // if user hasn't deposited yet
        let user_deposit_balance = user_profile
            .account_deposits
            .get(&market_id.clone())
            .unwrap_or_else(|| {
                panic!("User no have balance in token {}:", market_id.clone());
            });
        println!("user_deposit_balance: {}", user_deposit_balance);
        println!("amount: {}", &BigDecimal::from(amount));
        require!(
            user_deposit_balance >= &amount.0,
            "Not enough deposited balance"
        );
        let decreased_user_deposit = *user_deposit_balance - amount.0;
        user_profile
            .account_deposits
            .insert(market_id, decreased_user_deposit);
        self.user_profiles.insert(&user_id, &user_profile);
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

    pub fn calculate_pnl(
        &self,
        buy_token_price: WBigDecimal,
        sell_token_price: WBigDecimal,
        collateral_amount: WBigDecimal,
        leverage: U128,
    ) -> (bool, BigDecimal) {
        let borrow_amount =
            BigDecimal::from(buy_token_price) * BigDecimal::from(leverage.0) - BigDecimal::from(buy_token_price);
        let c_a = BigDecimal::from(collateral_amount) * BigDecimal::from(leverage.0);
        let div_value = borrow_amount / BigDecimal::from(sell_token_price)
            + BigDecimal::from(collateral_amount);
        let profit: bool;
        let result = if c_a > div_value {
            profit = true;
            c_a - div_value
        } else {
            profit = false;
            div_value - c_a
        };
        (profit, result)
    }

    #[allow(unused_variables)]
    pub fn get_liquidation_price(
        &self,
        sell_token_amount: U128,
        sell_token_price: U128,
        buy_token_price: U128,
        leverage: U128,
        borrow_fee: U128,
        swap_fee: U128,
    ) -> WBigDecimal {
        let sell_token = AccountId::new_unchecked("usdt.qa.nearlend.testnet".to_owned());
        let sell_token_price = self.get_price_by_token(sell_token);

        let buy_token = AccountId::new_unchecked("wnear.qa.nearlend.testnet".to_owned());
        let buy_token_price = self.get_price_by_token(buy_token);
        log!("buy_token_price {}", buy_token_price.0);

        let collateral_usd = BigDecimal::from(sell_token_amount) * BigDecimal::from(sell_token_price);
        let buy_amount = collateral_usd / BigDecimal::from(buy_token_price);

        let fee = BigDecimal::from_str("0.057").unwrap();
        let borrow_amount = collateral_usd * BigDecimal::from(leverage);

        (BigDecimal::from(buy_token_price) -  (collateral_usd - fee * borrow_amount) / buy_amount).into()
        // BigDecimal::from_str("2.41").unwrap().into()
    }

    #[payable]
    pub fn withdraw(&mut self, token_id: AccountId, amount: U128) {
        let balance = self.view_balance(env::signer_account_id(), token_id.clone());

        require!(
            balance.0 >= amount.0,
            format!("Account:{} not have enough balance", signer_account_id())
        );

        ext_token::ext(token_id.clone())
            .with_static_gas(self.terra_gas(5))
            .with_attached_deposit(1)
            .ft_transfer(
                signer_account_id(),
                amount,
                Some(format!(
                    "Withdraw from: {} amount: {}",
                    current_account_id(),
                    u128::try_from(amount).unwrap()
                )),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(self.terra_gas(2))
                    .with_attached_deposit(NO_DEPOSIT)
                    .withdraw_callback(token_id, amount),
            );
    }

    #[private]
    pub fn withdraw_callback(&mut self, token_id: AccountId, amount: U128) {
        require!(is_promise_success(), "Error transfer");

        self.decrease_user_deposit(token_id, signer_account_id(), amount);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use near_sdk::test_utils::test_env::{alice, bob};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .current_account_id(alice())
            .signer_account_id(bob())
            .predecessor_account_id("token_near".parse().unwrap())
            .block_index(1)
            .block_timestamp(1)
            .is_view(is_view)
            .build()
    }

    fn get_position() -> Position {
        //amount: 1000 * 10^24 USDT
        //leverage: 2.4 * 10^24
        //buy_token_price: 1.01 * 10^24
        //sell_token_price: 4.2 * 10^24
        Position {
            position_id: 0,
            active: true,
            p_type: PositionType::Long,
            sell_token: "usdc.nearland.testnet".parse().unwrap(),
            buy_token: "wnear.nearland.testnet".parse().unwrap(),
            collateral_amount: 1000 * 10_u128.pow(24),
            buy_token_price: 101 * 10_u128.pow(22),
            sell_token_price: 45 * 10_u128.pow(23),
            leverage: 24 * 10_u128.pow(23),
            borrow_amount: 404,
        }
    }

    fn get_position_examples() -> Position {
        //amount: 1 * 10^24 USDT
        //leverage: 3
        //buy_token_price: 3000 * 10^24
        //sell_token_price: 4100 * 10^24
        Position {
            position_id: 0,
            active: true,
            p_type: PositionType::Long,
            sell_token: "usdc.nearland.testnet".parse().unwrap(),
            buy_token: "wnear.nearland.testnet".parse().unwrap(),
            collateral_amount: 10_u128.pow(24),
            buy_token_price: 3000 * 10_u128.pow(24),
            sell_token_price: 4100 * 10_u128.pow(24),
            leverage: 3,
            borrow_amount: 404,
        }
    }

    #[test]
    fn test_pnl() {
        let context = get_context(false);
        testing_env!(context);
        let token_markets: Vec<(AccountId, AccountId)> = vec![
            (
                "usdt.nearland.testnet".parse().unwrap(),
                "usdt_market.nearland.testnet".parse().unwrap(),
            ),
            (
                "wnear.nearland.testnet".parse().unwrap(),
                "wnear_market.nearland.testnet".parse().unwrap(),
            ),
        ];
        let contract = Contract::new(token_markets);

        let position = get_position_examples();
        let result = contract.calculate_pnl(
            WBigDecimal::from(position.buy_token_price),
            WBigDecimal::from(position.sell_token_price),
            WBigDecimal::from(position.collateral_amount),
            WBigDecimal::from(position.leverage),
        );

        assert_eq!(WBigDecimal::from(result.1), U128(536585365853658536585366));
    }

    #[test]
    fn test_liquidation_price() {
        let context = get_context(false);
        testing_env!(context);
        let token_markets: Vec<(AccountId, AccountId)> = vec![
            (
                "usdt.nearland.testnet".parse().unwrap(),
                "usdt_market.nearland.testnet".parse().unwrap(),
            ),
            (
                "wnear.nearland.testnet".parse().unwrap(),
                "wnear_market.nearland.testnet".parse().unwrap(),
            ),
        ];
        let contract = Contract::new(token_markets);
        let position = get_position();

        let result = contract.get_liquidation_price(
            WBigDecimal::from(1),
            WBigDecimal::from(position.sell_token_price),
            WBigDecimal::from(position.buy_token_price),
            WBigDecimal::from(position.leverage),
            U128(5 * 10_u128.pow(6)),
            U128(3 * 10_u128.pow(1)),
        );

        assert_eq!(result, U128(215745429394269796120481938246454935552));
    }
}
