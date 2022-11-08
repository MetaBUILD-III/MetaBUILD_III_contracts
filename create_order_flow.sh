near call wnear.qa.v1.nearlend.testnet ft_transfer_call '{"receiver_id": "limit_orders.dmytrosh.testnet", "amount": "200000", "msg": "{\"Deposit\": {\"token\": \"wnear.qa.v1.nearlend.testnet\"}}"}' --accountId dmytrosh.testnet --depositYocto 1 --gas 300000000000000

near view limit_orders.dmytrosh.testnet balance_of '{"account_id": "dmytrosh.testnet", "token": "wnear.qa.v1.nearlend.testnet" }' &

wait

near call limit_orders.dmytrosh.testnet create_order '{"order_type": "Buy", "amount": "100000", "sell_token": "wnear.qa.v1.nearlend.testnet", "buy_token": "usdt.qa.v1.nearlend.testnet", "leverage": "1"}' --accountId dmytrosh.testnet --gas 300000000000000
