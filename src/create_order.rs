use crate::big_decimal::{BigDecimal, WBalance};
use crate::ref_finance::ext_ref_finance;
use crate::utils::NO_DEPOSIT;
use crate::utils::{ext_market, ext_token};
use crate::*;
use near_sdk::env::current_account_id;
use near_sdk::{ext_contract, is_promise_success, log, serde_json, Gas, PromiseResult};

const GAS_FOR_BORROW: Gas = Gas(180_000_000_000_000);

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn borrow_buy_token_callback(&self, amount: WBalance);

    fn get_pool_info_callback(
        &mut self,
        user: AccountId,
        amount: WBalance,
        amount_to_proceed: WBalance,
        order: Order,
    ) -> PromiseOrValue<WBalance>;

    fn add_liquidity_callback(
        &mut self,
        user: AccountId,
        amount: WBalance,
        order: Order,
    ) -> PromiseOrValue<Balance>;
}

#[near_bindgen]
impl Contract {
    pub fn create_order(
        &mut self,
        order_type: OrderType,
        amount: WBalance,
        sell_token: AccountId,
        buy_token: AccountId,
        leverage: U128,
    ) -> PromiseOrValue<WBalance> {
        let user = env::signer_account_id();

        require!(
            self.balance_of(user.clone(), sell_token.clone()) >= amount.0,
            "User doesn't have enough deposit to proceed this action"
        );

        let amount_to_proceed = if BigDecimal::from(leverage) > BigDecimal::one() {
            let borrow_token_amount = U128::from(
                BigDecimal::from(amount)
                    * self.calculate_xrate(buy_token.clone(), sell_token.clone())
                    * BigDecimal::from(leverage),
            );
            log!("borrowing amount {}", borrow_token_amount.0);

            self.borrow_buy_token(borrow_token_amount, buy_token.clone());

            // if we have borrowed some tokens we have to add to liquidity pool corresponding amount
            borrow_token_amount
        } else {
            amount
        };

        let order = Order {
            status: OrderStatus::Pending,
            order_type,
            amount: Balance::from(amount_to_proceed),
            sell_token: sell_token.clone(),
            buy_token: buy_token.clone(),
            leverage: BigDecimal::from(leverage),
            sell_token_price: self.view_price(sell_token.clone()),
            buy_token_price: self.view_price(buy_token.clone()),
            block: env::block_height(),
            lpt_id: "".to_string(),
        };

        self.get_pool_info_callback(user, amount, amount_to_proceed, order)
    }

    #[private]
    pub fn get_pool_info_callback(
        &mut self,
        user: AccountId,
        amount: WBalance,
        amount_to_proceed: WBalance,
        mut order: Order,
    ) -> PromiseOrValue<WBalance> {
        require!(
            is_promise_success(),
            "Some problem with pool on ref finance"
        );
        let pool_info = match env::promise_result(0) {
            PromiseResult::NotReady => unreachable!(),
            PromiseResult::Successful(val) => {
                if let Ok(pool) = near_sdk::serde_json::from_slice::<PoolInfo>(&val) {
                    pool
                } else {
                    panic!("Some problem with pool parsing.")
                }
            }
            PromiseResult::Failed => panic!("Ref finance not found pool"),
        };

        require!(
            pool_info.state == PoolState::Running,
            "Some problem with pool, please contact with ref finance to support."
        );

        // let mut left_point = pool_info.current_point as i32;

        // while left_point % pool_info.point_delta as i32 != 0 {
        //     left_point += 1;
        // }

        // let right_point = left_point + pool_info.point_delta as i32;

        // let point_delta = 40u64;
        let left_point = -11320i32;
        let right_point = -11280i32;

        let amount_x: WBalance = amount_to_proceed;
        let amount_y = U128::from(0);
        let min_amount_x = U128::from(0);
        let min_amount_y = U128::from(0);

        ext_token::ext(order.sell_token.clone())
            .with_static_gas(Gas::ONE_TERA * 35u64)
            .with_attached_deposit(near_sdk::ONE_YOCTO)
            .ft_transfer_call(
                self.ref_finance_account.clone(),
                amount_to_proceed,
                None,
                "\"Deposit\"".to_string(),
            )
            .and(
                ext_ref_finance::ext(self.ref_finance_account.clone())
                    .with_static_gas(Gas::ONE_TERA * 10u64)
                    .with_attached_deposit(NO_DEPOSIT)
                    .add_liquidity(
                        self.view_pair(&order.sell_token, &order.buy_token).pool_id,
                        left_point,
                        right_point,
                        amount_x,
                        amount_y,
                        min_amount_x,
                        min_amount_y,
                    ),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas::ONE_TERA * 2u64)
                    .with_attached_deposit(NO_DEPOSIT)
                    .add_liquidity_callback(user, amount, order),
            )
            .into()
    }

    #[private]
    pub fn add_liquidity_callback(
        &mut self,
        user: AccountId,
        amount: WBalance,
        mut order: Order,
    ) -> PromiseOrValue<WBalance> {
        require!(
            env::promise_results_count() == 2,
            "Contract expected 2 results on the callback"
        );
        match env::promise_result(0) {
            PromiseResult::NotReady => panic!("failed to deposit liquidity"),
            PromiseResult::Failed => panic!("failed to deposit liquidity"),
            _ => (),
        };

        self.decrease_balance(user.clone(), order.sell_token.clone(), amount.0);

        let lpt_id: String = match env::promise_result(1) {
            PromiseResult::Successful(result) => {
                serde_json::from_slice::<String>(&result).unwrap().into()
            }
            _ => panic!("failed to add liquidity"),
        };

        order.lpt_id = lpt_id;

        self.order_nonce += 1;
        let order_id = self.order_nonce;
        self.insert_order_for_user(&user, order, order_id);

        PromiseOrValue::Value(U128(0))
    }

    #[private]
    pub fn add_order(&mut self, account_id: AccountId, order: String) {
        self.order_nonce += 1;
        let order_id = self.order_nonce;
        let order = serde_json::from_str(order.as_str()).unwrap();
        self.insert_order_for_user(&account_id, order, order_id);
    }

    pub fn insert_order_for_user(&mut self, account_id: &AccountId, order: Order, order_id: u64) {
        let mut user_orders_by_id = self.orders.get(&account_id).unwrap_or_default();
        user_orders_by_id.insert(order_id, order);
        self.orders.insert(&account_id, &user_orders_by_id);
    }

    pub fn borrow_buy_token(&self, amount: U128, token: AccountId) {
        require!(
            env::prepaid_gas() >= GAS_FOR_BORROW,
            "Prepaid gas is not enough for borrow flow"
        );

        assert!(
            Balance::from(amount) > 0,
            "Amount should be a positive number"
        );

        let token_market = self.get_market_by(token);

        ext_market::ext(token_market)
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

    #[private]
    pub fn borrow_buy_token_callback(&self, amount: U128) {
        if !is_promise_success() {
            log!("{}", "Borrow has failed");
        }
    }
}
