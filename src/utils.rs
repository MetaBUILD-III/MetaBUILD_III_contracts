use near_sdk::{
    ext_contract, json_types::U128, AccountId
};

pub type WBalance = U128;

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