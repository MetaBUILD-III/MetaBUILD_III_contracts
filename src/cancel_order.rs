use crate::big_decimal::{BigDecimal, WBalance, WRatio};
use crate::*;
use near_sdk::env::{current_account_id, signer_account_id};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::{ext_contract, log, Gas};

type LptId = u64;

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

#[ext_contract(ref_finance)]
trait RefFinanceInterface {
    fn remove_liquidity(
        &mut self,
        lpt_id: LptId,
        amount: U128,
        min_amount_x: U128,
        min_amount_y: U128,
    ) -> (U128, U128);
}

#[ext_contract(ext_token)]
trait NEP141Token {
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: WBalance,
        memo: Option<String>,
        msg: String,
    );
}

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn remove_liquidity_callback(&self, order_id: U128, swap_fee: U128, price_impact: U128);
    fn swap_callback(&mut self, order_id: U128, swap_fee: U128, price_impact: U128);
}

#[near_bindgen]
impl Contract {
    fn cancel_order(&mut self, order_id: U128, swap_fee: U128, price_impact: U128) {
        let orders = self.orders.get(&signer_account_id()).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", signer_account_id());
        });

        let order = orders.get(&(order_id.0 as u64)).unwrap_or_else(|| {
            panic!("Order with id: {} not found", order_id.0);
        });

        //TODO: set real min_amount_x/min_amount_y
        let amount = 1;
        let min_amount_x = order.amount;
        let min_amount_y = 0;

        if order.status == OrderStatus::Pending {
            ref_finance::ext(order.buy_token.clone())
                .with_static_gas(Gas(10))
                .with_attached_deposit(1)
                .remove_liquidity(
                    self.pool_id,
                    U128(amount),
                    U128(min_amount_x),
                    U128(min_amount_y),
                )
                .then(
                    ext_self::ext(current_account_id())
                        .with_static_gas(Gas(20))
                        .with_attached_deposit(NO_DEPOSIT)
                        .remove_liquidity_callback(order_id, swap_fee, price_impact),
                );
        } else {
            self.remove_liquidity_callback(order_id, swap_fee, price_impact);
        }
    }

    #[private]
    pub fn remove_liquidity_callback(&self, order_id: U128, swap_fee: U128, price_impact: U128) {
        let orders = self.orders.get(&signer_account_id()).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", signer_account_id());
        });

        let order = orders.get(&(order_id.0 as u64)).unwrap_or_else(|| {
            panic!("Order with id: {} not found", order_id.0);
        });
        let buy_amount =
            BigDecimal::from(order.amount) * order.leverage * order.sell_token_price.value
                / order.buy_token_price.value;
        let min_amount = buy_amount * self.get_price(order.buy_token.clone()).value;
        let actions: Vec<Action> = vec![Action::Swap(SwapAction {
            pool_id: self.pool_id,
            token_in: order.buy_token.clone(),
            amount_in: Some(WRatio::from(buy_amount)),
            token_out: order.sell_token.clone(),
            min_amount_out: WRatio::from(min_amount),
        })];
        let action = TokenReceiverMessage::Execute {
            force: true,
            actions,
        };

        log!(
            "action {}",
            near_sdk::serde_json::to_string(&action).unwrap()
        );

        ext_token::ext(order.buy_token.clone())
            .with_static_gas(Gas(3))
            .with_attached_deposit(1)
            .ft_transfer_call(
                self.ref_finance_account.clone(),
                WRatio::from(buy_amount),
                Some("Deposit tokens".to_string()),
                near_sdk::serde_json::to_string(&action).unwrap(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(20))
                    .with_attached_deposit(NO_DEPOSIT)
                    .swap_callback(order_id, swap_fee, price_impact),
            );
    }

    pub fn swap_callback(&mut self, order_id: U128, swap_fee: U128, price_impact: U128) {
        let orders = self.orders.get(&signer_account_id()).unwrap_or_else(|| {
            panic!("Orders for account: {} not found", signer_account_id());
        });

        let order = orders.get(&(order_id.0 as u64)).unwrap_or_else(|| {
            panic!("Order with id: {} not found", order_id.0);
        });

        let market_id = self.tokens_markets.get(&order.sell_token).unwrap();
        let market_data = self.market_infos.get(&market_id).unwrap();
        let sell_amount =
            order.sell_token_price.value * BigDecimal::from(order.amount) * order.leverage;
        let pnl = self.calculate_pnl(signer_account_id(), order_id, market_data);

        let expect_amount = self.get_price(order.buy_token.clone()).value
            * sell_amount
            * BigDecimal::from(10_u128.pow(24) - swap_fee.0)
            * BigDecimal::from(10_u128.pow(24) - price_impact.0)
            / order.buy_token_price.value;
        if pnl.is_profit {
            let protocol_profit = expect_amount - sell_amount - BigDecimal::from(pnl.amount);

            let token_profit = self
                .protocol_profit
                .get(&order.sell_token)
                .unwrap_or_default();
            self.protocol_profit.insert(
                &order.sell_token,
                &(BigDecimal::from(token_profit) + protocol_profit),
            );
        }
    }
}
