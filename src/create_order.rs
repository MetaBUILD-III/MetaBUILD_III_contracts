use crate::big_decimal::{BigDecimal, WBalance};
use crate::ref_finance::ref_finance::ext;
use crate::ref_finance::{Action, SwapAction, TokenReceiverMessage};
use crate::utils::NO_DEPOSIT;
use crate::utils::{ext_market, ext_token};
use crate::*;
use near_sdk::env::current_account_id;
use near_sdk::{ext_contract, is_promise_success, log, serde_json, Gas, PromiseResult};

const GAS_FOR_BORROW: Gas = Gas(180_000_000_000_000);
const GAS_FOR_ADD_LIQUIDITY: Gas = Gas(200_000_000_000_000);

#[ext_contract(ext_self)]
trait ContractCallbackInterface {
    fn swap_callback(
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

        let min_amount_out = U128::from(
            BigDecimal::from(U128::from(amount_to_proceed))
                * self.calculate_xrate(buy_token.clone(), sell_token.clone()),
        );
        log!("min_amount_out {}", min_amount_out.0);

        let actions: Vec<Action> = vec![Action::Swap(SwapAction {
            pool_id: self.pool_id.clone(),
            token_in: buy_token.clone(),
            amount_in: Some(amount_to_proceed),
            token_out: sell_token.clone(),
            min_amount_out,
        })];

        let action = TokenReceiverMessage::Execute {
            force: true,
            actions,
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

        ext_token::ext(sell_token.clone())
            .with_static_gas(Gas(3))
            .with_attached_deposit(1)
            .ft_transfer_call(
                self.ref_finance_account.clone(),
                amount,
                Some("Deposit tokens".to_string()),
                near_sdk::serde_json::to_string(&action).unwrap(),
            )
            .then(
                ext_self::ext(current_account_id())
                    .with_static_gas(Gas(20))
                    .with_attached_deposit(NO_DEPOSIT)
                    .swap_callback(user, amount, order),
            )
            .into()
    }

    #[private]
    pub fn swap_callback(
        &mut self,
        user: AccountId,
        amount: WBalance,
        mut order: Order,
    ) -> PromiseOrValue<WBalance> {
        require!(is_promise_success(), "Token swap hasn't end successfully");

        self.decrease_balance(user.clone(), order.sell_token.clone(), amount.0);

        let left_point = 1;
        let right_point = 2;

        let amount_x = amount;
        let amount_y: WBalance = U128::from(0);
        let min_amount_x: U128 = amount;
        let min_amount_y: U128 = U128::from(0);

        // TODO set real parameters for calling add_liquidity on ref finance after deploying on testnet

        ext(self.ref_finance_account.clone())
            .with_static_gas(GAS_FOR_ADD_LIQUIDITY)
            .with_attached_deposit(NO_DEPOSIT)
            .add_liquidity(
                U128(12),
                left_point,
                right_point,
                amount_x,
                amount_y,
                min_amount_x,
                min_amount_y,
            );

        let lpt_id: String = match env::promise_result(0) {
            PromiseResult::NotReady => "".parse().unwrap(),
            PromiseResult::Failed => "".parse().unwrap(),
            PromiseResult::Successful(result) => {
                near_sdk::serde_json::from_slice::<String>(&result)
                    .unwrap()
                    .into()
            }
        };

        order.set_lpt_id(lpt_id);

        self.order_nonce += 1;
        let order_id = self.order_nonce;
        self.insert_order_for_user(&user, order, order_id);

        PromiseOrValue::Value(0.into())
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
            .borrow(amount);
    }
}
