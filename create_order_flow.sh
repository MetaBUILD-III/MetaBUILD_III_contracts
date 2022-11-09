near call usdt.qa.v1.nearlend.testnet ft_transfer_call '{"receiver_id": "limit_orders.dmytrosh.testnet", "amount": "10000000", "msg": "{\"Deposit\": {\"token\": \"usdt.qa.v1.nearlend.testnet\"}}"}' --accountId dmytrosh.testnet --depositYocto 1 --gas 300000000000000

near call dcl.ref-dev.testnet storage_deposit '{"account_id": "limit_orders.dmytrosh.testnet"}' --accountId dmytrosh.testnet --amount 1

near view limit_orders.dmytrosh.testnet balance_of '{"account_id": "dmytrosh.testnet", "token": "usdt.qa.v1.nearlend.testnet" }' &

wait

near call limit_orders.dmytrosh.testnet create_order '{"order_type": "Buy", "amount": "10000000", "sell_token": "usdt.qa.v1.nearlend.testnet", "buy_token": "wnear.qa.v1.nearlend.testnet", "leverage": "1"}' --accountId dmytrosh.testnet --gas 300000000000000

# make sure lpt id is valid
near view limit_orders.dmytrosh.testnet view_orders '{    "account_id":"dmytrosh.testnet",
                                                          "buy_token":"wnear.qa.v1.nearlend.testnet",
                                                          "sell_token":"usdt.qa.v1.nearlend.testnet"}'
