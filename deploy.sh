# login
#near login

# build & test
./build.sh && ./test.sh

# clean up previuos deployment
echo 'y' | near delete margin.nearlend.testnet nearlend.testnet

# create corresponding accoutns
near create-account margin.nearlend.testnet --masterAccount nearlend.testnet --initialBalance 20

# redeploy contracts
near deploy margin.nearlend.testnet --wasmFile ./res/mtrading.wasm