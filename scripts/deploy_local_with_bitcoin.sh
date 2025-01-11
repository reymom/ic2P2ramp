#!/bin/bash

./init_bitcoind.sh

./generate_env.sh local

dfx start --background --clean

DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd)"

shellcheck source=../.env
source "$DIR/../.env" || {
  echo "error while sourcing env file"
  exit
}

# Bitcoin Deployment
cargo build --release --target wasm32-unknown-unknown --package bitcoin_backend

candid-extractor target/wasm32-unknown-unknown/release/bitcoin_backend.wasm > bitcoin_backend/bitcoin_backend.did

dfx canister create --with-cycles 1_000_000_000_000 bitcoin_backend

dfx deploy bitcoin_backend --specified-id zhuzm-wqaaa-aaaap-qpk2q-cai --argument '(variant { regtest })'

# Backend Deployment
cargo build --release --target wasm32-unknown-unknown --package backend

candid-extractor target/wasm32-unknown-unknown/release/backend.wasm > backend/backend.did

dfx canister create --with-cycles 1_000_000_000_000 backend

dfx deploy backend --argument "(
  variant { 
    Reinstall = record {
      ecdsa_key_id = record {
        name = \"dfx_test_key\";
        curve = variant { secp256k1 };
      };
      chains = vec {
        record {
          chain_id = 11155111 : nat64;
          vault_manager_address = \"${CONTRACT_SEPOLIA}\";
          services = variant { EthSepolia = opt vec { variant { BlockPi } } };
          currency_symbol = \"ETH\";
        };
        record {
          chain_id = 84532 : nat64;
          vault_manager_address = \"${CONTRACT_BASE_SEPOLIA}\";
          services = variant {
            Custom = record {
              chainId = 84532 : nat64;
              services = vec {
                record { url = \"https://base-sepolia.infura.io/v3/${INFURA_API_KEY}\"; headers = null };
              };
            }
          };
          currency_symbol = \"ETH\";
        };
        record {
          chain_id = 11155420 : nat64;
          vault_manager_address = \"${CONTRACT_OP_SEPOLIA}\";
          services = variant {
            Custom = record {
              chainId = 11155420 : nat64;
              services = vec {
                record { url = \"https://opt-sepolia.g.alchemy.com/v2/${ALCHEMY_API_KEY}\"; headers = null };
              };
            }
          };
          currency_symbol = \"ETH\";
        };
        record {
          chain_id = 421614 : nat64;
          vault_manager_address = \"${CONTRACT_ARBITRUM_SEPOLIA}\";
          services = variant {
            Custom = record {
              chainId = 421614 : nat64;
              services = vec {
                record { url = \"https://arb-sepolia.g.alchemy.com/v2/${ALCHEMY_API_KEY}\"; headers = null };
              };
            }
          };
          currency_symbol = \"ETH\";
        };
        record {
          chain_id = 5003 : nat64;
          vault_manager_address = \"${CONTRACT_MANTLE_SEPOLIA}\";
          services = variant {
            Custom = record {
              chainId = 5003 : nat64;
              services = vec {
                record { url = \"https://rpc.ankr.com/mantle_sepolia\"; headers = null };
              };
            }
          };
          currency_symbol = \"MNT\";
        };
      };
      paypal = record {
        client_id = \"${PAYPAL_CLIENT_ID}\";
        client_secret = \"${PAYPAL_CLIENT_SECRET}\";
        api_url = \"api-m.sandbox.paypal.com\";
      };
      revolut = record {
        client_id = \"${REVOLUT_CLIENT_ID}\";
        api_url = \"https://sandbox-oba.revolut.com\";
        proxy_url = \"https://dc55-92-178-206-241.ngrok-free.app\";
        private_key_der = blob \"$(echo $(cat revolut_certs/private.key | base64 -w 0) | base64 --decode)\";
        kid = \"kid_0\";
        tan = \"test-jwk.s3.eu-west-3.amazonaws.com\";
      };
      proxy_url = \"https://ic2p2ramp.xyz\";
    }
  }
)"

dfx canister call bitcoin_backend register_runes '(vec {
    record { symbol = "🐕"; divisibility = 5 : nat8; cap = 100_000_000_000 : nat; premine = 100_000_000_000 : nat };
    record { symbol = "🤖"; divisibility = 5 : nat8; cap = 1_000_000_000 : nat; premine = 1_000_000_000 : nat };
    record { symbol = "🐸"; divisibility = 5 : nat8; cap = 20_814_270_000 : nat; premine = 20_790_000_000 : nat };
    record { symbol = "🧙"; divisibility = 0 : nat8; cap = 10_000 : nat; premine = 10_000 : nat };
})'