near call margin.nearlend.testnet set_price '{ "market_id": "wnear.qa.nearlend.testnet", "price": {"value": "5090000000000000000000000", "fraction_digits": 24} }' --accountId nearlend.testnet
near view margin.nearlend.testnet get_price_by_token '{ "token_id": "wnear.qa.nearlend.testnet" }'

near call margin.nearlend.testnet set_pool_id '{"pool_id": "1744"}' --accountId margin.nearlend.testnet --gas 300000000000000

near call margin.nearlend.testnet close_position '{"position_id": "0"}' --accountId nearlend.testnet --gas 300000000000000
