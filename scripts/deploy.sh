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

# Local network
dfx ledger fabricate-cycles --icp 10000 --canister $(dfx identity get-wallet)

# Transfer cycles from identity to wallet
dfx cycles top-up --network ic $(dfx identity get-wallet --network ic) 1_000_000_000_000

dfx deps pull && dfx deps init evm_rpc --argument '(record { nodesInSubnet = 28 })' && dfx deps deploy

# Build the canister
cargo build --release --target wasm32-unknown-unknown --package backend

# Create the canister with specified cycles
dfx canister create --with-cycles 1_000_000_000_000 backend --ic

# Install the canister with initial state arguments
dfx canister install --wasm target/wasm32-unknown-unknown/release/backend.wasm backend --mode reinstall --argument "(
  record {
    ecdsa_key_id = record {
      name = \"dfx_test_key\";
      curve = variant { secp256k1 };
    };
    chains = vec {
      record {
        chain_id = 11155111 : nat64;
        vault_manager_address = \"0x42ad57ab757ea55960f7d9805d82fa818683096b\";
        services = variant { EthSepolia = opt vec { variant { Alchemy } } };
      };
      record {
        chain_id = 84532 : nat64;
        vault_manager_address = \"0xfa29381958DD8a2dD86246FC0Ab2932972640580\";
        services = variant {
          Custom = record {
            chainId = 84532 : nat64;
            services = vec {
              record { url = \"https://base-sepolia.g.alchemy.com/v2/${ALCHEMY_API_KEY}\"; headers = null };
            };
          }
        };
      };
      record {
        chain_id = 11155420 : nat64;
        vault_manager_address = \"0x9025e74D23384f664CfEB07F1d8ABd19570758B5\";
        services = variant {
          Custom = record {
            chainId = 11155420 : nat64;
            services = vec {
              record { url = \"https://opt-sepolia.g.alchemy.com/v2/${ALCHEMY_API_KEY}\"; headers = null };
            };
          }
        };
      };
      record {
        chain_id = 2442 : nat64;
        vault_manager_address = \"0x9025e74D23384f664CfEB07F1d8ABd19570758B5\";
        services = variant {
          Custom = record {
            chainId = 2442 : nat64;
            services = vec {
              record { url = \"https://polygonzkevm-cardona.g.alchemy.com/v2/${ALCHEMY_API_KEY}\"; headers = null };
            };
          }
        };
      };
    };
    paypal = record {
      client_id = \"${PAYPAL_CLIENT_ID}\";
      client_secret = \"${PAYPAL_CLIENT_SECRET}\";
      api_url = \"https://api-m.sandbox.paypal.com\";
    };
    revolut = record {
      client_id = \"${REVOLUT_CLIENT_ID}\";
      api_url = \"https://sandbox-oba.revolut.com\";
      proxy_url = \"https://dc55-92-178-206-241.ngrok-free.app\";
      private_key_der = blob \"$(echo $(cat revolut_certs/private.key | base64 -w 0) | base64 --decode)\";
      kid = \"kid_0\";
      tan = \"test-jwk.s3.eu-west-3.amazonaws.com\";
    };
  },
)"

# Wait for a few seconds to allow the EVM address to be generated
sleep 3

# Retrieve and export the EVM address
export EVM_ADDRESS=$(dfx canister call backend get_evm_address | awk -F'"' '{print $2}')
echo "EVM_ADDRESS: $EVM_ADDRESS"

# Check the status of the canister to verify it's running
dfx canister status backend
