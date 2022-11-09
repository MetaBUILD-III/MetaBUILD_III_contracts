# login
#near login

# build & test
mkdir -p res && ./build.sh && ./test.sh

# clean up previuos deployment
echo 'y' | near delete limit_orders.v1.nearlend.testnet v1.nearlend.testnet

# create corresponding accoutns
near create-account limit_orders.v1.nearlend.testnet --masterAccount v1.nearlend.testnet --initialBalance 10

# redeploy contracts
near deploy limit_orders.v1.nearlend.testnet \
  --wasmFile ./res/limit_orders.wasm \
  --initFunction 'new_with_config' \
  --initArgs '{
        "owner_id":"limit_orders.v1.nearlend.testnet",
        "oracle_account_id":"limit_orders_oracle.v1.nearlend.testnet"
    }'

# register limit orders on tokens
near call wnear.qa.v1.nearlend.testnet storage_deposit '{"account_id": "limit_orders.v1.nearlend.testnet"}' --accountId limit_orders.v1.nearlend.testnet --amount 0.25 &
near call usdt.qa.v1.nearlend.testnet storage_deposit '{"account_id": "limit_orders.v1.nearlend.testnet"}' --accountId limit_orders.v1.nearlend.testnet --amount 0.25 &
wait

# add supported pairs
near call limit_orders.v1.nearlend.testnet add_pair '{
        "pair_data": {
            "sell_ticker_id": "nUSDT",
            "sell_token": "usdt.qa.v1.nearlend.testnet",
            "sell_token_market": "usdt_market.qa.v1.nearlend.testnet",
            "buy_ticker_id": "near",
            "buy_token": "wnear.qa.v1.nearlend.testnet"
        }
    }' --accountId limit_orders.v1.nearlend.testnet &

near call limit_orders.v1.nearlend.testnet add_pair '{
        "pair_data": {
            "sell_ticker_id": "near",
            "sell_token": "wnear.qa.v1.nearlend.testnet",
            "sell_token_market": "wnear_market.qa.v1.nearlend.testnet",
            "buy_ticker_id": "nUSDT",
            "buy_token": "usdt.qa.v1.nearlend.testnet"
        }
    }' --accountId limit_orders.v1.nearlend.testnet &

wait
near view limit_orders.v1.nearlend.testnet view_supported_pairs '{}'

# add mock prices
near call limit_orders.v1.nearlend.testnet update_or_insert_price '{
    "token_id":"usdt.qa.v1.nearlend.testnet",
    "price":{
        "ticker_id":"nUSDT",
        "value":"1.01"
    }
}' --accountId limit_orders.v1.nearlend.testnet

near call limit_orders.v1.nearlend.testnet update_or_insert_price '{
    "token_id":"wnear.qa.v1.nearlend.testnet",
    "price":{
        "ticker_id":"near",
        "value":"3.07"
    }
}' --accountId limit_orders.v1.nearlend.testnet &

wait
near view limit_orders.v1.nearlend.testnet view_price '{"token_id":"usdt.qa.v1.nearlend.testnet"}'
near view limit_orders.v1.nearlend.testnet view_price '{"token_id":"wnear.qa.v1.nearlend.testnet"}'

# add mock orders
near call limit_orders.v1.nearlend.testnet add_order '{
        "account_id":"tommylinks.testnet",
        "order":"{\"status\":\"Executed\",\"order_type\":\"Buy\",\"amount\":1000000100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"2.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"4.22\"},\"block\":103930916,\"lpt_id\":\"1\"}"
    }' --accountId limit_orders.v1.nearlend.testnet &

near call limit_orders.v1.nearlend.testnet add_order '{
        "account_id":"tommylinks.testnet",
        "order":"{\"status\":\"Pending\",\"order_type\":\"Buy\",\"amount\":1000001100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.5\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"1.01\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.01\"},\"block\":103930917,\"lpt_id\":\"2\"}"
    }' --accountId limit_orders.v1.nearlend.testnet &

near call limit_orders.v1.nearlend.testnet add_order '{
        "account_id":"tommylinks.testnet",
        "order":"{\"status\":\"Canceled\",\"order_type\":\"Buy\",\"amount\":2000001100000000000000000000,\"sell_token\":\"usdt.qa.v1.nearlend.testnet\",\"buy_token\":\"wnear.qa.v1.nearlend.testnet\",\"leverage\":\"1.0\",\"sell_token_price\":{\"ticker_id\":\"USDT\",\"value\":\"0.99\"},\"buy_token_price\":{\"ticker_id\":\"WNEAR\",\"value\":\"3.99\"},\"block\":103930918,\"lpt_id\":\"3\"}"
    }' --accountId limit_orders.v1.nearlend.testnet &

wait

near view limit_orders.v1.nearlend.testnet view_orders '{
    "account_id":"tommylinks.testnet",
    "buy_token":"wnear.qa.v1.nearlend.testnet",
    "sell_token":"usdt.qa.v1.nearlend.testnet"
}'
