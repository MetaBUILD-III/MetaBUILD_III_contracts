near call usdt.qa.v1.nearlend.testnet ft_transfer_call '{"receiver_id": "limit_orders.v1.nearlend.testnet", "amount": "10000000", "msg": "{\"Deposit\": {\"token\": \"usdt.qa.v1.nearlend.testnet\"}}"}' --accountId nearlend.testnet --depositYocto 1 --gas 300000000000000

near call dcl.ref-dev.testnet storage_deposit '{"account_id": "limit_orders.v1.nearlend.testnet"}' --accountId nearlend.testnet --amount 1

near view limit_orders.v1.nearlend.testnet balance_of '{"account_id": "nearlend.testnet", "token": "usdt.qa.v1.nearlend.testnet" }' &

wait

near call limit_orders.v1.nearlend.testnet create_order '{"order_type": "Buy", "amount": "10000000", "sell_token": "usdt.qa.v1.nearlend.testnet", "buy_token": "wnear.qa.v1.nearlend.testnet", "leverage": "1"}' --accountId nearlend.testnet --gas 300000000000000

# make sure lpt id is valid
near view limit_orders.v1.nearlend.testnet view_orders '{    "account_id":"nearlend.testnet ",
                                                          "buy_token":"wnear.qa.v1.nearlend.testnet",
                                                          "sell_token":"usdt.qa.v1.nearlend.testnet"}'
