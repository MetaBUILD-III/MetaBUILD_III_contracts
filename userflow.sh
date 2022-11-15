near call  usdt.qa.v1.nearlend.testnet ft_transfer_call '{"receiver_id": "limit_orders.v1.nearlend.testnet", "amount": "2000000000000000000000000000", "msg": "{\"Deposit\": {\"token\": \"usdt.qa.v1.nearlend.testnet\"}}"}' --accountId nearlend.testnet --depositYocto 1 --gas 32000000000000

near view limit_orders.v1.nearlend.testnet balance_of '{"account_id": "nearlend.testnet", "token": "usdt.qa.v1.nearlend.testnet" }' 

# amount = 1000.0
# leverage = 1.0
near call limit_orders.v1.nearlend.testnet create_order '{
    "order_type": "Buy",
    "amount": "1000000000000000000000000000",
    "sell_token": "usdt.qa.v1.nearlend.testnet",
    "buy_token": "wnear.qa.v1.nearlend.testnet",
    "leverage": "1000000000000000000000000" 
}' --accountId nearlend.testnet --gas 300000000000000

near call limit_orders.v1.nearlend.testnet create_order '{
    "order_type": "Buy",
    "amount": "1000000000000000000000000000",
    "sell_token": "usdt.qa.v1.nearlend.testnet",
    "buy_token": "wnear.qa.v1.nearlend.testnet",
    "leverage": "1000000000000000000000000" 
}' --accountId nearlend.testnet --gas 300000000000000


near view limit_orders.v1.nearlend.testnet view_orders '{
    "account_id":"nearlend.testnet",
    "buy_token":"wnear.qa.v1.nearlend.testnet",
    "sell_token":"usdt.qa.v1.nearlend.testnet"
}'

near view dcl.ref-dev.testnet get_pool '{
    "pool_id": "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000"
}'

near view dcl.ref-dev.testnet get_pool '{
    "pool_id": "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000"
}'

near view dcl.ref-dev.testnet get_liquidity '{
    "lpt_id": "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#193"
}'


# for i in {185..200}
# do
#     near call dcl.ref-dev.testnet remove_liquidity '{
#         "lpt_id": "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000#'$i'",
#         "amount": "14209047472819294933719294",
#         "min_amount_x": "0",
#         "min_amount_y": "0"
#     }' --accountId limit_orders.v1.nearlend.testnet --gas 300000000000000 &"
# done
# wait

# swap_fee 0.002 = 0.2%
# price_impact 0.05 = 5%
near call limit_orders.v1.nearlend.testnet cancel_order '{
    "order_id": "4",
    "swap_fee": "2000000000000000000000",
    "price_impact": "50000000000000000000000"
}' --accountId nearlend.testnet --gas 300000000000000