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

sleep 20

dfx canister call backend test_set_paypal_token

dfx canister log backend

dfx stop