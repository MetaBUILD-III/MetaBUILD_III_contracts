# login
#near login

# build & test
./build.sh && ./test.sh

# clean up previuos deployment
echo 'y' | near delete margin.nearlend.testnet nearlend.testnet

# create corresponding accoutns
near create-account margin.nearlend.testnet --masterAccount nearlend.testnet --initialBalance 20

# redeploy contracts
near deploy margin.nearlend.testnet \
    --wasmFile ./res/mtrading.wasm \
    --initFunction 'new' \
    --initArgs '{
        "tokens_markets": [["usdt.nearland.testnet","usdt_market.qa.nearlend.testnet"], ["wnear.qa.nearlend.testnet", "wnear_market.qa.nearlend.testnet"]]
    }'
