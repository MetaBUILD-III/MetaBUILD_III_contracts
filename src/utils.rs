use near_sdk::AccountId;

fn get_ref_finance_account() -> AccountId {
    "ref-exchange.testnet".parse().unwrap()
}