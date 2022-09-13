near call usdt.qa.nearlend.testnet mint '{
    "account_id": "nearlend.testnet",
    "amount": "1500000000000000000000000000"
}' --accountId nearlend.testnet


near call --depositYocto 1 --gas 300000000000000 usdt.qa.nearlend.testnet ft_transfer_call '{"receiver_id": "usdt_market.qa.nearlend.testnet", "amount": "1500000000000000000000000000", "msg":"\"Deposit\""}' --accountId nearlend.testnet

near view margin.nearlend.testnet view_balance '{ "user": "nearlend.testnet", "market": "usdt.qa.nearlend.testnet" }'

near call margin.nearlend.testnet set_price '{ "market_id": "usdt.qa.nearlend.testnet", "price": {"value": "1010000000000000000000000", "fraction_digits": 24} }' --accountId nearlend.testnet
near view margin.nearlend.testnet get_price_by_token '{ "token_id": "usdt.qa.nearlend.testnet" }'


near call margin.nearlend.testnet set_price '{ "market_id": "wnear.qa.nearlend.testnet", "price": {"value": "4590000000000000000000000", "fraction_digits": 24} }' --accountId nearlend.testnet
near view margin.nearlend.testnet get_price_by_token '{ "token_id": "wnear.qa.nearlend.testnet" }'

# near call margin.nearlend.testnet open_position '{"sell_token": "usdt.qa.nearlend.testnet", "sell_token_amount": "1000000000000000000000000000", "buy_token": "wnear.qa.nearlend.testnet", "leverage": "1500000000000000000000000"}' --accountId nearlend.testnet --gas 300000000000000

near view margin.nearlend.testnet view_user_positions '{ "user": "nearlend.testnet", "market": "usdt.qa.nearlend.testnet" }'
