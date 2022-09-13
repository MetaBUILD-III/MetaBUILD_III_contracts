#!/bin/bash

# extend whitelisted tokens
near call margin1.nearlend.testnet add_available_tokens '{"tokens": ["usdt.qa.nearlend.testnet", "wnear.qa.nearlend.testnet"]}' --account-id margin1.nearlend.testnet --amount 1 --gas 300000000000000

near call ref-finance-101.testnet storage_deposit '{"account_id": "margin1.nearlend.testnet"}' --accountId nearlend.testnet --amount 0.25


# create simple pool
near call margin1.nearlend.testnet set_pool_id '{"tokens": ["usdt.qa.nearlend.testnet", "wnear.qa.nearlend.testnet"], "fee": 25}' --account-id margin1.nearlend.testnet --amount 0.3 --gas 300000000000000

# create zero balances
near call margin1.nearlend.testnet register_tokens '{"tokens":  ["usdt.qa.nearlend.testnet", "wnear.qa.nearlend.testnet"]}' --account-id margin1.nearlend.testnet --amount 1 --gas 300000000000000

# fund tokens (before it contract must have balance in deposit tokens)
near call usdt.qa.nearlend.testnet storage_deposit '{"account_id": "ref-finance-101.testnet"}' --accountId nearlend.testnet --amount 0.25

near call margin1.nearlend.testnet deposit_tokens '{"token_id": "usdt.qa.nearlend.testnet", "amount": "100"}' --account-id margin1.nearlend.testnet --amount 0.25 --gas 300000000000000


near call wnear.qa.nearlend.testnet storage_deposit '{"account_id": "ref-finance-101.testnet"}' --accountId nearlend.testnet --amount 0.25
near call margin1.nearlend.testnet deposit_tokens '{"token_id": "wnear.qa.nearlend.testnet", "amount": "100"}' --account-id margin1.nearlend.testnet --amount 0.25 --gas 300000000000000

# add liquidity
near call margin1.nearlend.testnet add_liquidity '{ "amounts": "10" }' --account-id margin1.nearlend.testnet --amount 1 --gas 300000000000000

# swap
near call margin1.nearlend.testnet execute_position '{ "token_in": "wnear.qa.nearlend.testnet", "amount_in": "1000000000000000000000000", "token_out": "usdt.qa.nearlend.testnet", "min_amount_out": "4129490024018948000000000"}' --account-id margin1.nearlend.testnet --gas 300000000000000