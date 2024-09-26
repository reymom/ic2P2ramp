#!/bin/bash

DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd)"

shellcheck source=../.env
source "$DIR/../.env" || {
  echo "error while sourcing env file"
  exit
}

# cargo build --release --target wasm32-unknown-unknown --package backend

# candid-extractor target/wasm32-unknown-unknown/release/backend.wasm > backend/backend.did

# dfx start --background --clean

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
      token_symbol = opt \"LICP\";
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
      token_symbol = opt \"ckTESTBTC\";
      token_name = opt \"Chain key testnet Bitcoin\";
    }
  })
"

dfx deploy internet_identity

dfx deps deploy xrc

dfx deps deploy evm_rpc

dfx generate backend

# dfx_test_key, test_key_1
# api-m.paypal.com, api-m.sandbox.paypal.com
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

dfx canister call backend register_icp_tokens '(vec { "ryjl3-tyaaa-aaaaa-aaaba-cai"; "mc6ru-gyaaa-aaaar-qaaaq-cai" })'
dfx canister call backend register_evm_tokens '(11155111 : nat64, vec {
    record { "0x878bfCfbB8EAFA8A2189fd616F282E1637E06bcF"; 18 : nat8; "USD"; opt "Custom USDT deployed by me" }
})'
dfx canister call backend register_evm_tokens '(84532 : nat64, vec {
    record { "0x036CbD53842c5426634e7929541eC2318f3dCF7e"; 6 : nat8; "USD"; opt "Sepolia USDC" }
})'

# dfx deploy frontend --mode reinstall

# export ACCOUNT_ID=$(dfx ledger account-id --of-principal x43o3-z4337-mle53-vdvne-poc44-i7e66-rr34k-3sdep-uebye-i4r3t-7qe)

# export TO_PRINCIPAL=2n6h4-allra-4vkzg-2g45r-cnzrr-ivz7f-i6442-toets-lkrsh-4psdz-aqe
# export TO_SUBACCOUNT="null"
# export AMOUNT="5_250_000_000"
# export FEE="10_000"

# dfx canister call ryjl3-tyaaa-aaaaa-aaaba-cai icrc1_transfer \
# '(record {
#     to = record {
#         owner = principal "'$TO_PRINCIPAL'";
#         subaccount = '$TO_SUBACCOUNT';
#     };
#     fee = opt '$FEE';
#     memo = null;
#     from_subaccount = null;
#     created_at_time = null;
#     amount = '$AMOUNT';
# })'

# export AMOUNT="500_000_000"
# export FEE="10"

# dfx canister call mc6ru-gyaaa-aaaar-qaaaq-cai icrc1_transfer \
# '(record {
#     to = record {
#         owner = principal "'$TO_PRINCIPAL'";
#         subaccount = '$TO_SUBACCOUNT';
#     };
#     fee = opt '$FEE';
#     memo = null;
#     from_subaccount = null;
#     created_at_time = null;
#     amount = '$AMOUNT';
# })'
