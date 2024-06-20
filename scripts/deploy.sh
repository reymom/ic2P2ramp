#!/bin/bash

dfx stop
dfx=$(lsof -t -i:4943)
# Check if any PIDs were found
if [ -z "$dfx" ]; then
    echo "dfx not running."
else
    # Kill the processes
    kill $dfx && echo "Terminating running dfx instance."
    sleep 3
fi

DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd)"

# shellcheck source=../.env
source "$DIR/../.env" || {
  echo "error while sourcing env file"
  exit
}

# Start the local replica in the background
dfx start --clean --background

dfx ledger fabricate-cycles --icp 10000 --canister $(dfx identity get-wallet)

dfx deps pull && dfx deps init evm_rpc --argument '(record { nodesInSubnet = 28 })' && dfx deps deploy

# Build the canister
cargo build --release --target wasm32-unknown-unknown --package backend

# Create the canister with specified cycles
dfx canister create --with-cycles 5_000_000_000_000 backend

# Install the canister with initial state arguments
dfx canister install --wasm target/wasm32-unknown-unknown/release/backend.wasm backend --mode reinstall --argument '(
  record {
    ecdsa_key_id = record {
      name = "test_key_1";
      curve = variant { secp256k1 };
    };
    rpc_services = vec {
      record {
        chain_id = 11155111 : nat64;
        service = variant {
          EthSepolia = opt vec {
            variant { Alchemy }
          }
        }
      },
      record {
        chain_id = 5003 : nat64;
        service = variant {
          Custom = record {
            chainId = 5003 : nat64;
            services = vec { record { url = "https://rpc.sepolia.mantle.xyz"; headers = null } };
          }
        }
      }
    };
    vault_manager_addresses = vec {
      record { 11155111 : nat64; "0x42ad57ab757ea55960f7d9805d82fa818683096b" };
      record { 5003 : nat64; "0xdB976eCC0c95Ea84d7bB7249920Fcc73392783F5" };
    };
    client_id = "${CLIENT_ID}";
    client_secret = "${CLIENT_SECRET}";
  },
)'

# Wait for a few seconds to allow the EVM address to be generated
sleep 3

# Retrieve and export the EVM address
export EVM_ADDRESS=$(dfx canister call backend get_evm_address | awk -F'"' '{print $2}')
echo "EVM_ADDRESS: $EVM_ADDRESS"

# Check the status of the canister to verify it's running
dfx canister status backend
