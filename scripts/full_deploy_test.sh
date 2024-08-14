#!/bin/bash

DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd)"

# shellcheck source=../.env
source "$DIR/../.env" || {
  echo "error while sourcing env file"
  exit
}

cargo build --release --target wasm32-unknown-unknown --package backend

candid-extractor target/wasm32-unknown-unknown/release/backend.wasm > backend/backend.did

dfx start --background --clean

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

dfx generate backend

dfx deploy backend --argument "(
  record {
    ecdsa_key_id = record {
      name = \"dfx_test_key\";
      curve = variant { secp256k1 };
    };
    chains = vec {};
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

export ACCOUNT_ID=$(dfx ledger account-id --of-principal zkp5d-qz6bw-3glxn-l2635-w5fyq-rxjb2-27zcp-deyty-ayjfn-kvbqh-5ae)

export TO_PRINCIPAL=dtuky-kq5aj-vxqk7-m4kmi-kv4ld-zvsno-pdufp-5lwim-hnowi-amamz-iqe
export TO_SUBACCOUNT="null"
export AMOUNT="200_000_000"
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

export AMOUNT="100_000_000"
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