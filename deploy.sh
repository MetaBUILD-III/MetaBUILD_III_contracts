# login
#near login

# build & test
mkdir -p res && ./build.sh && ./test.sh

CONTRACT_ADDRESS=leverage_omomo.testnet

# clean up previuos deployment
# echo 'y' | near delete ${CONTRACT_ADDRESS} v1.nearlend.testnet

# create corresponding accoutns
# near create-account ${CONTRACT_ADDRESS} --masterAccount v1.nearlend.testnet --initialBalance 10

# redeploy contracts
near deploy ${CONTRACT_ADDRESS} \
    --wasmFile ./res/limit_orders.wasm
#   --wasmFile ./res/limit_orders.wasm \
#   --initFunction 'new_with_config' \
#   --initArgs '{
#         "owner_id":"'${CONTRACT_ADDRESS}'",
#         "oracle_account_id":"limit_orders_oracle.v1.nearlend.testnet"
#     }'

# register limit orders on tokens
near call wnear.qa.v1.nearlend.testnet storage_deposit '{"account_id": "'${CONTRACT_ADDRESS}'"}' --accountId ${CONTRACT_ADDRESS} --amount 0.25 &
near call usdt.qa.v1.nearlend.testnet storage_deposit '{"account_id": "'${CONTRACT_ADDRESS}'"}' --accountId ${CONTRACT_ADDRESS} --amount 0.25 &
wait

# add supported pairs
near call ${CONTRACT_ADDRESS} add_pair '{
        "pair_data": {
            "sell_ticker_id": "USDt",
            "sell_token": "usdt.qa.v1.nearlend.testnet",
            "sell_token_market": "usdt_market.qa.v1.nearlend.testnet",
            "buy_ticker_id": "near",
            "buy_token": "wnear.qa.v1.nearlend.testnet",
            "pool_id": "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000"
        }
    }' --accountId ${CONTRACT_ADDRESS} &

near call ${CONTRACT_ADDRESS} add_pair '{
        "pair_data": {
            "sell_ticker_id": "near",
            "sell_token": "wnear.qa.v1.nearlend.testnet",
            "sell_token_market": "wnear_market.qa.v1.nearlend.testnet",
            "buy_ticker_id": "USDt",
            "buy_token": "usdt.qa.v1.nearlend.testnet",
            "pool_id": "usdt.qa.v1.nearlend.testnet|wnear.qa.v1.nearlend.testnet|2000"
        }
    }' --accountId ${CONTRACT_ADDRESS} &

# near view ${CONTRACT_ADDRESS} view_supported_pairs '{}'

# add mock prices
near call ${CONTRACT_ADDRESS} update_or_insert_price '{
    "token_id":"usdt.qa.v1.nearlend.testnet",
    "price":{
        "ticker_id":"USDt",
        "value":"1.01"
    }
}' --accountId ${CONTRACT_ADDRESS} &

near call ${CONTRACT_ADDRESS} update_or_insert_price '{
    "token_id":"wnear.qa.v1.nearlend.testnet",
    "price":{
        "ticker_id":"near",
        "value":"1.83"
    }
}' --accountId ${CONTRACT_ADDRESS} &

near view ${CONTRACT_ADDRESS} view_price '{"token_id":"usdt.qa.v1.nearlend.testnet"}'
near view ${CONTRACT_ADDRESS} view_price '{"token_id":"wnear.qa.v1.nearlend.testnet"}'

# add mock orders
# near call ${CONTRACT_ADDRESS} add_order '{
#         "account_id":"tommylinks.testnet",
#         "order":"{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":1000000100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"block\":103930916,\"lpt_id\":\"1\"}"
#     }' --accountId ${CONTRACT_ADDRESS} &

# near call ${CONTRACT_ADDRESS} add_order '{
#         "account_id":"tommylinks.testnet",
#         "order":"{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000001100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.01\"},\"block\":103930917,\"lpt_id\":\"2\"}"
#     }' --accountId ${CONTRACT_ADDRESS} &

# near call ${CONTRACT_ADDRESS} add_order '{
#         "account_id":"tommylinks.testnet",
#         "order":"{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000001100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"0.99\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.99\"},\"block\":103930918,\"lpt_id\":\"3\"}"
#     }' --accountId ${CONTRACT_ADDRESS} &

# near call ${CONTRACT_ADDRESS} add_order '{
#         "account_id":"nearlend.testnet",
#         "order":"{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":1000000100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"block\":103930916,\"lpt_id\":\"1\"}"
#     }' --accountId ${CONTRACT_ADDRESS} &

# near call ${CONTRACT_ADDRESS} add_order '{
#         "account_id":"nearlend.testnet",
#         "order":"{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000001100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.01\"},\"block\":103930917,\"lpt_id\":\"2\"}"
#     }' --accountId ${CONTRACT_ADDRESS} &


# near call ${CONTRACT_ADDRESS} add_order '{
#         "account_id":"nearlend.testnet",
#         "order":"{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000001100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"0.99\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.99\"},\"block\":103930918,\"lpt_id\":\"3\"}"
#     }' --accountId ${CONTRACT_ADDRESS} &


# setup pool
near call dcl.ref-dev.testnet storage_deposit '{"account_id": "'${CONTRACT_ADDRESS}'"}' --accountId nearlend.testnet --amount 1 &

near call ${CONTRACT_ADDRESS} add_token_market '{"token_id": "wnear.qa.v1.nearlend.testnet", "market_id": "wnear_market.qa.v1.nearlend.testnet"}' --account_id ${CONTRACT_ADDRESS} &
near call ${CONTRACT_ADDRESS} add_token_market '{"token_id": "usdt.qa.v1.nearlend.testnet", "market_id": "usdt_market.qa.v1.nearlend.testnet"}' --account_id ${CONTRACT_ADDRESS} &

near call usdt_market.qa.v1.nearlend.testnet set_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ADDRESS}'" }' --accountId shared_admin.testnet
near view usdt_market.qa.v1.nearlend.testnet get_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ADDRESS}'" }'

near call controller.qa.v1.nearlend.testnet set_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ADDRESS}'" }' --accountId controller.qa.v1.nearlend.testnet
near view controller.qa.v1.nearlend.testnet get_eligible_to_borrow_uncollateralized_account '{ "account": "'${CONTRACT_ADDRESS}'" }'

wait
