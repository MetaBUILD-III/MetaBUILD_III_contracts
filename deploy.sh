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
        "oracle_account_id":"test_ac_oracle.testnet"
    }'

near call wnear.qa.v1.nearlend.testnet mint '{"account_id": "limit_orders.v1.nearlend.testnet", "amount": "10000000"}' --accountId limit_orders.v1.nearlend.testnet
near call usdt.qa.v1.nearlend.testnet mint '{"account_id": "limit_orders.v1.nearlend.testnet", "amount": "10000000"}' --accountId limit_orders.v1.nearlend.testnet

near call limit_orders.v1.nearlend.testnet add_pair '{
        "pair_data": {
            
            "sell_ticker_id": "usdt",
            "sell_token": "usdt.qa.v1.nearlend.testnet",
            "sell_token_market": "usdt_market.qa.v1.nearlend.testnet",
            "buy_ticker_id": "wnear",
            "buy_token": "wnear.qa.v1.nearlend.testnet"
        }
    }' --accountId limit_orders.v1.nearlend.testnet

near call limit_orders.v1.nearlend.testnet add_pair '{
        "pair_data": {
            "sell_ticker_id": "wnear",
            "sell_token": "wnear.qa.v1.nearlend.testnet",
            "sell_token_market": "wnear_market.qa.v1.nearlend.testnet",
            "buy_ticker_id": "usdt",
            "buy_token": "usdt.qa.v1.nearlend.testnet"
        }
    }' --accountId limit_orders.v1.nearlend.testnet

near view limit_orders.v1.nearlend.testnet view_supported_pairs '{}'
