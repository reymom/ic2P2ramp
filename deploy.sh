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

# Start the local replica in the background
dfx start --clean --background

dfx ledger fabricate-cycles --icp 10000 --canister $(dfx identity get-wallet)

dfx deps pull && dfx deps init evm_rpc --argument '(record { nodesInSubnet = 28 })' && dfx deps deploy

# Build the canister
cargo build --release --target wasm32-unknown-unknown --package backend

# Create the canister with specified cycles
dfx canister create --with-cycles 10_000_000_000_000 backend

# Install the canister with initial state arguments
dfx canister install --wasm target/wasm32-unknown-unknown/release/backend.wasm backend --argument '(
  record {
    ecdsa_key_id = record {
      name = "dfx_test_key";
      curve = variant { secp256k1 };
    };
    rpc_services = variant {
      Custom = record {
        chainId = 5003 : nat64;
        services = vec { record { url = "https://rpc.sepolia.mantle.xyz"; headers = null } };
      }
    };
    rpc_service = variant {
      Custom = record {
        url = "https://rpc.sepolia.mantle.xyz";
        headers = null;
      }
    };
    block_tag = variant { Latest = null };
    client_id = "Ab_E80t7BM4rNxj7trOAlRz_UmpEqPHANABmFUzD-7Zj-iiUI9nhkRilop_2lWKoWTE_bfEFiXV33mHb";
    client_secret = "EPLGQdKtjDZ3-42STiwCVUaoEz0-9r_Wc2R8PUJFlMiXMSiwot8vb1FGYPGEhaYhmB7wVio1DApKdGvW";
  },
)'

# Wait for a few seconds to allow the EVM address to be generated
sleep 3

# Retrieve and export the EVM address
export EVM_ADDRESS=$(dfx canister call backend get_evm_address | awk -F'"' '{print $2}')
echo "EVM_ADDRESS: $EVM_ADDRESS"

# Check the status of the canister to verify it's running
dfx canister status backend
