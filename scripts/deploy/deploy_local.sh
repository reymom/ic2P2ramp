#!/bin/bash

# dfx stop
# dfx=$(lsof -t -i:4943)
# # Check if any PIDs were found
# if [ -z "$dfx" ]; then
#     echo "dfx not running."
# else
#     # Kill the processes
#     kill $dfx && echo "Terminating running dfx instance."
#     sleep 3
# fi

dfx start --background --clean

DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd)"

shellcheck source=../.env
source "$DIR/../.env" || {
  echo "error while sourcing env file"
  exit
}

# ---------
# Deploy our Bitcoin and Solana backend canisters
# ---------

cargo build --release --target wasm32-unknown-unknown --package bitcoin_backend
candid-extractor target/wasm32-unknown-unknown/release/bitcoin_backend.wasm > backend/bitcoin/bitcoin_backend.did

dfx deploy bitcoin_backend --specified-id zhuzm-wqaaa-aaaap-qpk2q-cai --argument "(
    variant { 
        Reinstall = record { 
            network = variant { regtest }; 
            proxy_url = \"https://ic2p2ramp.xyz\";
            unisat = record {
                api_url = \"open-api-testnet4.unisat.io\";
                api_key = \"${UNISAT_API_KEY}\";
            }; 
        }
    }
)"

dfx deploy sol_rpc

dfx canister call sol_rpc updateApiKeys "(vec {
  record { variant { AlchemyDevnet }; opt \"$ALCHEMY_KEY\" };
  record { variant { AnkrDevnet }; opt \"$ANKR_KEY\" };
})"

cargo build --release --target wasm32-unknown-unknown --package solana_backend
candid-extractor target/wasm32-unknown-unknown/release/solana_backend.wasm > backend/solana/solana_backend.did

dfx deploy solana_backend --specified-id u6s2n-gx777-77774-qaaba-cai --argument "(
    variant { 
        Reinstall = record {
            sol_rpc_canister_id = opt principal \"tghme-zyaaa-aaaar-qarca-cai\";
            ed25519_key_name = variant { LocalDevelopment };
            network = variant { Devnet }; 
            proxy_url = \"https://ic2p2ramp.xyz\";
        }
    }
)"

# --------
# Deploy icramp backend canister dependencies 
# --------

dfx identity use minter
export MINTER_ACCOUNT_ID=$(dfx ledger account-id)

dfx identity use default
export DEFAULT_ACCOUNT_ID=$(dfx ledger account-id)

dfx deploy icp_ledger_canister --argument "
  (variant {
    Init = record {
      minting_account = \"$MINTER_ACCOUNT_ID\";
      initial_values = vec {
        record {
          \"$DEFAULT_ACCOUNT_ID\";
          record {
            e8s = 10_000_000_000 : nat64;
          };
        };
      };
      send_whitelist = vec {};
      transfer_fee = opt record {
        e8s = 10_000 : nat64;
      };
      token_symbol = opt \"ICP\";
      token_name = opt \"Local ICP\";
    }
  })
"

dfx deploy ckbtc_ledger_canister_testnet --argument "
  (variant {
    Init = record {
      minting_account = \"$(dfx ledger account-id --of-principal ml52i-qqaaa-aaaar-qaaba-cai)\";
      initial_values = vec {
        record {
          \"$DEFAULT_ACCOUNT_ID\";
          record {
            e8s = 10_000_000_000 : nat64;
          };
        };
      };
      send_whitelist = vec {};
      transfer_fee = opt record {
        e8s = 10 : nat64;
      };
      token_symbol = opt \"BTC\";
      token_name = opt \"Chain key testnet Bitcoin\";
    }
  })
"

dfx deps pull
dfx deps deploy xrc
dfx deps init evm_rpc --argument '(record {})' && dfx deps deploy evm_rpc

# --------------------------
# Deploy icramp main backend
# --------------------------

cargo build --release --target wasm32-unknown-unknown --package icramp_backend
candid-extractor target/wasm32-unknown-unknown/release/icramp_backend.wasm > backend/icramp/icramp_backend.did

# dfx_test_key, test_key_1
# api-m.paypal.com, api-m.sandbox.paypal.com
dfx deploy icramp_backend --argument "(
  variant { 
    Reinstall = record {
      canister_ids = record {
        bitcoin_backend_id = \"zhuzm-wqaaa-aaaap-qpk2q-cai\";
        solana_backend_id = \"u6s2n-gx777-77774-qaaba-cai\";
      };
      ecdsa_key_id = record {
        name = \"dfx_test_key\";
        curve = variant { secp256k1 };
      };
      chains = vec {
        record {
          chain_id = 11155111 : nat64;
          vault_manager_address = \"${CONTRACT_SEPOLIA}\";
          services = variant { EthSepolia = opt vec { variant { Alchemy } } };
          currency_symbol = \"ETH\";
        };
        record {
          chain_id = 84532 : nat64;
          vault_manager_address = \"${CONTRACT_BASE_SEPOLIA}\";
          services = variant {
            Custom = record {
              chainId = 84532 : nat64;
              services = vec {
                record { url = \"https://base-sepolia.g.alchemy.com/v2/${ALCHEMY_API_KEY}\"; headers = null };
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
)"

dfx canister call icramp_backend register_icp_tokens '(vec { "ryjl3-tyaaa-aaaaa-aaaba-cai"; "mc6ru-gyaaa-aaaar-qaaaq-cai" })'
dfx canister call icramp_backend register_evm_tokens '(11155111 : nat64, vec {
    record { "0x1c7D4B196Cb0C7B01d743Fbc6116a902379C7238"; 6 : nat8; "USD"; opt "Sepolia Official USDC" };
    record { "0x08210F9170F89Ab7658F0B5E3fF39b0E03C594D4"; 6 : nat8; "EUR"; opt "Sepolia Official EURC" };
    record { "0x878bfCfbB8EAFA8A2189fd616F282E1637E06bcF"; 18 : nat8; "USD"; opt "Custom USDT deployed by me" };
})'
dfx canister call icramp_backend register_evm_tokens '(84532 : nat64, vec {
    record { "0x036CbD53842c5426634e7929541eC2318f3dCF7e"; 6 : nat8; "USD"; opt "Base Sepolia Official USDC" };
    record { "0x808456652fdb597867f38412077A9182bf77359F"; 6 : nat8; "EUR"; opt "Sepolia Official EURC" };
})'
dfx canister call icramp_backend register_evm_tokens '(11155420 : nat64, vec {
    record { "0x5fd84259d66Cd46123540766Be93DFE6D43130D7"; 6 : nat8; "USD"; opt "Optimism Sepolia Official USDC" };
})'
dfx canister call icramp_backend register_evm_tokens '(421614 : nat64, vec {
    record { "0x75faf114eafb1BDbe2F0316DF893fd58CE46AA4d"; 6 : nat8; "USD"; opt "Arbitrum Sepolia Official USDC" };
})'
dfx canister call bitcoin_backend register_runes '(vec {
    record { id = "66593:594"; name = "DOG•GO•TO•THE•MOON"; symbol = "🐕"; divisibility = 6 : nat8; cap = 0 : nat; premine = 1_000_000_000 : nat };
    record { id = "73393:191"; name = "UNCOMMON•GOODS"; symbol = "⧉"; divisibility = 0 : nat8; cap = 10_000 : nat; premine = 0 : nat };
})'
dfx canister call solana_backend register_tokens '(vec { record { "FxoGGtuyjfVybdA3X5WgxzNhjvSN73R5zqPYg3on8hwE"; "KONG"; "KONG" } })'

dfx generate icramp_backend
dfx generate bitcoin_backend
dfx generate solana_backend

cd frontend && npm run build && cd .. && dfx deploy frontend --mode reinstall --yes

# Fund the frontend's II with some of our locally deployed tokens

export TO_PRINCIPAL="dvbrj-gc3mc-56aem-lxs4s-yq2sj-5xryx-zgkrd-zk3xu-glhtj-wpotk-tae"
export TO_SUBACCOUNT="null"
export AMOUNT="2_500_000_000"
export FEE="10_000"

dfx canister call ryjl3-tyaaa-aaaaa-aaaba-cai icrc1_transfer \
'(record {
    to = record {
        owner = principal "'$TO_PRINCIPAL'";
        subaccount = '$TO_SUBACCOUNT';
    };
    fee = opt '$FEE';
    memo = null;
    from_subaccount = null;
    created_at_time = null;
    amount = '$AMOUNT';
})'

export AMOUNT="500_000_000"
export FEE="10"

dfx canister call mc6ru-gyaaa-aaaar-qaaaq-cai icrc1_transfer \
'(record {
    to = record {
        owner = principal "'$TO_PRINCIPAL'";
        subaccount = '$TO_SUBACCOUNT';
    };
    fee = opt '$FEE';
    memo = null;
    from_subaccount = null;
    created_at_time = null;
    amount = '$AMOUNT';
})'
