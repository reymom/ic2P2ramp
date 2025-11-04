#!/bin/bash

./generate_env.sh sandbox

DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd)"

shellcheck source=../.env.sandbox
source "$DIR/../.env.sandbox" || {
  echo "error while sourcing env file"
  exit
}

# -------
# Bitcoin
# -------

cargo build --release --target wasm32-unknown-unknown --package bitcoin_backend
candid-extractor target/wasm32-unknown-unknown/release/bitcoin_backend.wasm > backend/bitcoin/bitcoin_backend.did

# Might be necessary
# dfx cycles top-up --ic $(dfx identity get-wallet --ic) 1_000_000_000_000
dfx canister create --with-cycles 1_000_000_000_000 bitcoin_backend --ic

dfx deploy bitcoin_backend --argument "(
    variant { 
        Reinstall = record { 
            network = variant { testnet }; 
            proxy_url = \"https://icramp.info\";
            unisat = record {
                api_url = \"open-api-testnet4.unisat.io\";
                api_key = \"${UNISAT_API_KEY}\";
            }; 
        }
    }
)" --ic

# ------
# Solana
# ------

# dfx deploy sol_rpc --ic

# dfx canister call sol_rpc updateApiKeys "(vec {
#   record { variant { AlchemyDevnet }; opt \"$SOL_ALCHEMY_KEY\" };
#   record { variant { AnkrDevnet }; opt \"$SOL_ANKR_KEY\" };
# })" --ic

cargo build --release --target wasm32-unknown-unknown --package solana_backend
candid-extractor target/wasm32-unknown-unknown/release/solana_backend.wasm > backend/solana/solana_backend.did

dfx canister create --with-cycles 1_000_000_000_000 solana_backend --ic
dfx deploy solana_backend --argument "(
    variant {
        Reinstall = record {
            sol_rpc_canister_id = opt principal \"tghme-zyaaa-aaaar-qarca-cai\";
            ed25519_key_name = variant { MainnetTestKey1 };
            network = variant { Devnet };
            proxy_url = \"https://icramp.info\";
        }
    }
)" --ic

# -------
# backend
# -------

# dfx deps pull && dfx deps init evm_rpc --argument '(record { nodesInSubnet = 28 })' && dfx deps deploy

cargo build --release --target wasm32-unknown-unknown --package icramp_backend
candid-extractor target/wasm32-unknown-unknown/release/icramp_backend.wasm > backend/icramp/icramp_backend.did

dfx canister create --with-cycles 1_000_000_000_000 icramp_backend --ic

dfx deploy icramp_backend --argument "(
  variant { 
    Reinstall = record {
      canister_ids = record {
        solana_backend_id = \"i7jas-faaaa-aaaap-qqdva-cai\";
        bitcoin_backend_id = \"iwklo-tiaaa-aaaap-qqduq-cai\";
      };
      ecdsa_key_id = record {
        name = \"test_key_1\";
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
      proxy_url = \"https://icramp.info\";
      ordiscan = record {
        api_url = \"api.ordiscan.com\";
        api_key = \"${ORDISCAN_API_KEY}\";
      };
      unisat = record {
        api_url = \"open-api-testnet4.unisat.io\";
        api_key = \"${UNISAT_API_KEY}\";
      };
    }
  }
)" --ic

# configurations
dfx canister call icramp_backend register_icp_tokens '(vec {
    "ryjl3-tyaaa-aaaaa-aaaba-cai"; 
    "lkwrt-vyaaa-aaaaq-aadhq-cai";
    "2ouva-viaaa-aaaaq-aaamq-cai";
    "mxzaz-hqaaa-aaaar-qaada-cai";
})' --ic

dfx canister call icramp_backend register_evm_tokens '(11155111 : nat64, vec {
    record { "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"; 6 : nat8; "USD"; opt "Sepolia Official USDC" };
    record { "0x08210F9170F89Ab7658F0B5E3fF39b0E03C594D4"; 6 : nat8; "EUR"; opt "Sepolia Official EURC" };
    record { "0x878bfCfbB8EAFA8A2189fd616F282E1637E06bcF"; 18 : nat8; "USD"; opt "Custom USDT deployed by me" };
})' --ic
dfx canister call icramp_backend register_evm_tokens '(84532 : nat64, vec {
    record { "0x036CbD53842c5426634e7929541eC2318f3dCF7e"; 6 : nat8; "USD"; opt "Base Sepolia Official USDC" };
    record { "0x808456652fdb597867f38412077A9182bf77359F"; 6 : nat8; "EUR"; opt "Sepolia Official EURC" };
})' --ic
dfx canister call icramp_backend register_evm_tokens '(11155420 : nat64, vec {
    record { "0x5fd84259d66Cd46123540766Be93DFE6D43130D7"; 6 : nat8; "USD"; opt "Optimism Sepolia Official USDC" };
})' --ic
dfx canister call icramp_backend register_evm_tokens '(421614 : nat64, vec {
    record { "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d"; 6 : nat8; "USD"; opt "Arbitrum Sepolia Official USDC" };
})' --ic

dfx canister call bitcoin_backend register_runes '(vec {
    record { id = "66593:594"; name = "DOG•GO•TO•THE•MOON"; symbol = "🐕"; divisibility = 6 : nat8; cap = 0 : nat; premine = 1_000_000_000 : nat };
    record { id = "73393:191"; name = "UNCOMMON•GOODS"; symbol = "⧉"; divisibility = 0 : nat8; cap = 10_000 : nat; premine = 0 : nat };
})' --ic

dfx canister call solana_backend register_tokens '(vec {
  record { "FxoGGtuyjfVybdA3X5WgxzNhjvSN73R5zqPYg3on8hwE"; "KONG"; "KONG" };
  record { "HbA6BgBmA3X6X8jtts5X2ZiJXXxZQKDbQR4s5XCD82pr"; "BONK"; "BONK"};
})' --ic

dfx generate icramp_backend
dfx generate bitcoin_backend
dfx generate solana_backend

# dfx canister create --with-cycles 1_000_000_000_000 frontend --ic
cd frontend && npm run build && cd .. && dfx deploy frontend --mode reinstall --ic
