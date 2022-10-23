use crate::*;
use near_sdk::ext_contract;

pub const NO_DEPOSIT: Balance = 0;

#[ext_contract(ext_token)]
pub trait NEP141Token {
    fn ft_transfer_call(
        &mut self,
        receiver_id: AccountId,
        amount: WBalance,
        memo: Option<String>,
        msg: String,
    );

    fn ft_transfer(&mut self, receiver_id: AccountId, amount: WBalance, memo: Option<String>);
}
