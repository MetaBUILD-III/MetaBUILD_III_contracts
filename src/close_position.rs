use crate::*;
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
    fn swap_callback(
        &mut self,
        min_amount_out: WBalance,
        position: Position,
    ) -> PromiseOrValue<Balance>;
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
        let positions = self
            .positions
            .get(&signer_account_id())
            .unwrap_or_else(|| panic!("Positions for account: {} not found.", signer_account_id()));
        let position = positions
            .get(&position_id.0)
            .unwrap_or_else(|| panic!("Position with id: {} not found.", position_id.0));

        require!(position.active, "Position not active.");

        // TODO Receive min_amount_out (from UI?)
        let min_amount_out = U128::from(
            Ratio::from(U128::from(position.borrow_amount))
                * self.calculate_xrate(position.buy_token.clone(), position.sell_token.clone()),
        );
        log!("min_amount_out {}", min_amount_out.0);

        self.execute_position(position.clone(), min_amount_out)
    }

    fn execute_position(
        &mut self,
        position: Position,
        min_amount_out: U128,
    ) -> PromiseOrValue<Balance> {
        let actions: Vec<Action> = vec![Action::Swap(SwapAction {
            pool_id: self.pool_id,
            token_in: position.buy_token.clone(),
            amount_in: Some(position.borrow_amount.into()),
            token_out: position.sell_token.clone(),
            min_amount_out,
        })];

        let action = TokenReceiverMessage::Execute {
            force: true,
            actions,
        };

        log!(
            "action {}",
            near_sdk::serde_json::to_string(&action).unwrap()
        );

        ext_token::ext(position.buy_token.clone())
            .with_static_gas(Gas(3))
            .with_attached_deposit(1)
            .ft_transfer_call(
                REF_FINANCE.parse().unwrap(),
                position.borrow_amount.into(),
                Some("Deposit tokens".to_string()),
                near_sdk::serde_json::to_string(&action).unwrap(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(20))
                    .with_attached_deposit(NO_DEPOSIT)
                    .swap_callback(min_amount_out, position),
            )
            .into()
    }

    #[private]
    pub fn swap_callback(
        &mut self,
        min_amount_out: WBalance,
        position: Position,
    ) -> PromiseOrValue<WBalance> {
        require!(is_promise_success(), "Some problem with swap tokens");

        // let market_id = self
        //     .tokens_markets
        //     .get(&position.sell_token.clone())
        //     .unwrap();
        // let market_data = self.get_market_data(position.sell_token.clone(), market_id.clone());
        // let borrow_fee = market_data.borrow_rate_ratio.0;
        // let swap_fee = self.exchange_fee(amount.0);
        // let fee = borrow_fee + swap_fee;
        // self.decrease_user_deposit(market_id.clone(), signer_account_id(), U128(fee));

        let sell_token_price = self.get_price_by_token(position.sell_token.clone());
        let _pnl = self.calculate_pnl(
            U128(position.buy_token_price),
            sell_token_price,
            U128(position.collateral_amount),
            U128(position.leverage),
        );
        self.increase_user_deposit(
            position.sell_token.clone(),
            signer_account_id(),
            U128::from(min_amount_out.0 - position.borrow_amount),
        );

        // if pnl.0 {
        //     let fee_amount = WRatio::from(position.collateral_amount);
        //     self.decrease_user_deposit(
        //         position.sell_token.clone(),
        //         signer_account_id(),
        //         fee_amount,
        //     );
        // }

        let mut position = position;
        position.active = false;

        let mut positions = self.positions.get(&signer_account_id()).unwrap();
        positions.insert(position.position_id, position);
        self.positions.insert(&signer_account_id(), &positions);

        PromiseOrValue::Value(0.into())
    }

    #[private]
    pub fn set_pool_id(&mut self, pool_id: U128) {
        self.pool_id = pool_id.0 as u64;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    
    use near_sdk::test_utils::test_env::{alice};
    use near_sdk::test_utils::VMContextBuilder;
    use near_sdk::{testing_env, VMContext};

    fn get_context(is_view: bool) -> VMContext {
        VMContextBuilder::new()
            .current_account_id("margin.nearland.testnet".parse().unwrap())
            .signer_account_id(alice())
            .predecessor_account_id("usdt_market.qa.nearland.testnet".parse().unwrap())
            .block_index(1)
            .block_timestamp(1)
            .is_view(is_view)
            .build()
    }

    #[test]
    fn test_close_position() {
        let context = get_context(false);
        testing_env!(context);
        let token1: AccountId = "usdt.qa.nearlend.testnet".parse().unwrap();
        let market1: AccountId = "usdt_market.qa.nearland.testnet".parse().unwrap();
        let token2: AccountId = "wnear.qa.nearland.testnet".parse().unwrap();
        let market2: AccountId = "wnear_market.qa.nearland.testnet".parse().unwrap();

        let token_markets: Vec<(AccountId, AccountId)> = vec![
            (token1.clone(), market1),
            (token2, market2),
        ];
        let mut contract = Contract::new(token_markets);
        let position_id = U128(1);

        //user deposit amount
        contract.increase_user_deposit(token1.clone(), alice(), U128(1000000000000000000000000000));
        let price_token1: Price = Price {
            value: U128(100 * 10_u128.pow(24)),
            fraction_digits: 1,
        };
        let price_token2: Price = Price {
            value: U128(100 * 10_u128.pow(24)),
            fraction_digits: 1,
        };
        contract.set_price(token1.clone(), price_token1);
        contract.set_price("wnear.qa.nearland.testnet".parse().unwrap(), price_token2);

        //open position
        contract.open_position(
            token1.clone(),
            U128(1000000000000000000000000000),
            "wnear.qa.nearland.testnet".parse().unwrap(),
            U128(2000000000000000000000000),
        );
        assert_eq!(1, contract.positions.len());

        //set market_data
        let market_data = MarketData {
            total_supplies: U128(100 * 10_u128.pow(24)),
            total_borrows: U128(100 * 10_u128.pow(24)),
            total_reserves: U128(10_u128.pow(24)),
            exchange_rate_ratio: U128(10_u128.pow(24)),
            interest_rate_ratio: U128(10_u128.pow(24)),
            borrow_rate_ratio: U128(10_u128.pow(24)),
        };
        let mut market = LookupMap::new(b"market_data".to_vec());
        market.insert(&token1.clone(), &market_data);
        contract.markets_data.insert(&token1, &market);

        let mut position = contract.get_position(position_id);

        //set all fee's
        contract.set_exchange_fee(U128(2 * 10_u128.pow(24)));
        contract.set_referral_fee(U128(4 * 10_u128.pow(24)));
        contract.set_total_fee(U128(3 * 10_u128.pow(24)));

        // swap callback
        let amount = 100;
        let market_id = contract.tokens_markets.get(&position.sell_token).unwrap();
        let borrow_fee = market_data.borrow_rate_ratio.0;
        let swap_fee = contract.exchange_fee(amount);
        let fee = borrow_fee + swap_fee;
        println!(
            "market_id: {}, borrow_fee: {}, swap_fee: {}, fee: {}",
            market_id, borrow_fee, swap_fee, fee
        );

        contract.decrease_user_deposit(token1.clone(), alice(), U128(fee));
        let user_profile = contract.user_profiles.get(&alice()).unwrap();
        let account_deposit = user_profile.account_deposits.get(&token1).unwrap();
        println!("account_deposit after decrease fee: {:?}", account_deposit);

        let sell_token_price = contract.get_price_by_token(position.sell_token.clone());
        let pnl = contract.calculate_pnl(
            WRatio::from(position.buy_token_price),
            sell_token_price,
            WRatio::from(position.collateral_amount),
            WRatio::from(position.leverage),
        );
        println!("pnl: {:?}", pnl);
        let result = *account_deposit - WRatio::from(pnl.1).0;
        if pnl.0 {
            let fee_amount = Ratio::from(position.collateral_amount) / Ratio::from(10_u128);
            contract.decrease_user_deposit(token1.clone(), alice(), WRatio::from(fee_amount));
            contract.increase_user_deposit(token1.clone(), alice(), WRatio::from(pnl.1));
        } else {
            contract.decrease_user_deposit(token1.clone(), alice(), WRatio::from(pnl.1));
        }
        position.active = false;

        //update position
        let mut positions = contract.positions.get(&signer_account_id()).unwrap();
        positions.insert(position_id.0, position);
        contract.positions.insert(&signer_account_id(), &positions);

        //finish check
        let position = contract.get_position(U128(1));
        assert_eq!(false, position.active);

        let user_profile = contract.user_profiles.get(&alice()).unwrap();
        let account_deposit = user_profile.account_deposits.get(&token1).unwrap();
        println!("account_deposit: {:?}", account_deposit);
        assert_eq!(&result, account_deposit);
    }
}
