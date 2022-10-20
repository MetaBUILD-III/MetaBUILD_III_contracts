# login
#near login

# build & test
./build.sh && ./test.sh

# clean up previuos deployment
echo 'y' | near delete contract.mtrading_cl.testnet mtrading_cl.testnet

# create corresponding accoutns
near create-account contract.mtrading_cl.testnet --masterAccount mtrading_cl.testnet --initialBalance 20

# redeploy contracts
near deploy contract.mtrading_cl.testnet \
    --wasmFile ./res/mtradingcl.wasm \
    --initFunction 'new_with_config' \
    --initArgs '{"owner_id":"contract.mtrading_cl.testnet", "oracle_account_id":"test_ac_oracle.testnet"}'
