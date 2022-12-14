# login
#near login

# build & test
./build.sh && ./test.sh

# clean up previuos deployment
echo 'y' | near delete margin1.nearlend.testnet nearlend.testnet

# create corresponding accoutns
near create-account margin1.nearlend.testnet --masterAccount nearlend.testnet --initialBalance 20

# redeploy contracts
near deploy margin1.nearlend.testnet \
    --wasmFile ./res/mtrading.wasm \
    --initFunction "new" \
    --initArgs '{"markets": ["usdt.qa.nearlend.testnet", "wnear.qa.nearlend.testnet"]}'