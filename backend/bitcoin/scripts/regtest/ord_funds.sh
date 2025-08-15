#!/bin/bash

# explicitly create the ord wallet, will fail if it already exists but it's okay
./scripts/regtest/ord_wallet.sh create

# extract ord wallet address
ORD_ADDRESS=$(./scripts/regtest/ord_wallet.sh receive | jq -r '.addresses[0]')

echo $ORD_ADDRESS

# send funds to the ord wallet address
docker compose exec bitcoind bitcoin-cli -regtest -rpcwallet=testwallet sendtoaddress "$ORD_ADDRESS" 1

# mine a block to confirm the transaction
docker compose exec bitcoind bitcoin-cli -regtest -rpcwallet=testwallet generatetoaddress 1 $(docker compose exec bitcoind bitcoin-cli -regtest -rpcwallet=testwallet getnewaddress)
