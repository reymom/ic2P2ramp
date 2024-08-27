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

dfx deploy evm_rpc

dfx generate backend

dfx deploy backend --argument "(
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
  }
)"

dfx canister call backend register_icp_tokens '(vec { "ryjl3-tyaaa-aaaaa-aaaba-cai"; "mc6ru-gyaaa-aaaar-qaaaq-cai" })'

# export ACCOUNT_ID=$(dfx ledger account-id --of-principal x43o3-z4337-mle53-vdvne-poc44-i7e66-rr34k-3sdep-uebye-i4r3t-7qe)

# export TO_PRINCIPAL=7befc-xxqta-lqnm5-r6vfg-vfpss-mubwb-mosuw-wnhhj-qpfvf-67la2-hqe
# export TO_SUBACCOUNT="null"
# export AMOUNT="200_000_000"
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

# export AMOUNT="100_000_000"
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