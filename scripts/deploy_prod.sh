#!/bin/bash

DIR="$(cd "$( dirname "${BASH_SOURCE[0]}" )" >/dev/null 2>&1 && pwd)"

shellcheck source=../.env.production
source "$DIR/../.env.production" || {
  echo "error while sourcing env file"
  exit
}

# Might be necessary
# dfx ledger fabricate-cycles --icp 10000 --canister $(dfx identity get-wallet --ic)
# dfx cycles top-up --ic $(dfx identity get-wallet --ic) 1_000_000_000_000

cargo build --release --target wasm32-unknown-unknown --package backend

dfx canister create --with-cycles 1_000_000_000_000 backend_prod --ic

dfx deploy backend_prod --argument "(
  variant { 
    Reinstall = record {
      ecdsa_key_id = record {
        name = \"key_1\";
        curve = variant { secp256k1 };
      };
      chains = vec {
        record {
          chain_id = 1 : nat64;
          vault_manager_address = \"${CONTRACT_MAINNET}\";
          services = variant { EthMainnet = opt vec { variant { Alchemy } } };
          currency_symbol = \"ETH\";
        };
        record {
          chain_id = 8453 : nat64;
          vault_manager_address = \"${CONTRACT_BASE}\";
          services = variant {
            Custom = record {
              chainId = 8453 : nat64;
              services = vec {
                record { url = \"https://base-mainnet.g.alchemy.com/v2/${ALCHEMY_API_KEY}\"; headers = null };
              };
            }
          };
          currency_symbol = \"ETH\";
        };
        record {
          chain_id = 10 : nat64;
          vault_manager_address = \"${CONTRACT_OP}\";
          services = variant {
            Custom = record {
              chainId = 10 : nat64;
              services = vec {
                record { url = \"https://opt-mainnet.g.alchemy.com/v2/${ALCHEMY_API_KEY}\"; headers = null };
              };
            }
          };
          currency_symbol = \"ETH\";
        };
      };
      paypal = record {
        client_id = \"${PAYPAL_CLIENT_ID}\";
        client_secret = \"${PAYPAL_CLIENT_SECRET}\";
        api_url = \"api-m.paypal.com\";
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
)" --ic

# configurations
dfx canister call backend_prod register_icp_tokens '(vec { 
    "ryjl3-tyaaa-aaaaa-aaaba-cai"; 
    "lkwrt-vyaaa-aaaaq-aadhq-cai";
    "2ouva-viaaa-aaaaq-aaamq-cai";
    "mxzaz-hqaaa-aaaar-qaada-cai";
})' --ic
dfx canister call backend_prod register_evm_tokens '(1 : nat64, vec {
    record { "0xdAC17F958D2ee523a2206206994597C13D831ec7"; 6 : nat8; "USDT"; null };
    record { "0xa0b86991c6218b36c1d19d4a2e9eb0ce3606eb48"; 6 : nat8; "USDC"; null };
    record { "0x1aBaEA1f7C830bD89Acc67eC4af516284b1bC33c"; 6 : nat8; "EURC"; null };
    record { "0x6B175474E89094C44Da98b954EedeAC495271d0F"; 18 : nat8; "DAI"; null };
    record { "0x95aD61b0a150d79219dCF64E1E6Cc01f0B64C4cE"; 18 : nat8; "SHIB"; null };
})' --ic
dfx canister call backend_prod register_evm_tokens '(8453 : nat64, vec {
    record { "0xfde4C96c8593536E31F229EA8f37b2ADa2699bb2"; 6 : nat8; "USDT"; null };
    record { "0x833589fCD6eDb6E08f4c7C32D4f71b54bdA02913"; 6 : nat8; "USDC"; null };
    record { "0x60a3e35cc302bfa44cb288bc5a4f316fdb1adb42"; 6 : nat8; "EURC"; null };
    record { "0x50c5725949A6F0c72E6C4a641F24049A917DB0Cb"; 6 : nat8; "DAI"; null };
})' --ic
dfx canister call backend_prod register_evm_tokens '(10 : nat64, vec {
    record { "0x94b008aA00579c1307B0EF2c499aD98a8ce58e58"; 6 : nat8; "USDT"; null };
    record { "0x0b2c639c533813f4aa9d7837caf62653d097ff85"; 6 : nat8; "USDC"; null };
    record { "0xDA10009cBd5D07dd0CeCc66161FC93D7c9000da1"; 18 : nat8; "DAI"; null };
    record { "0x4200000000000000000000000000000000000042"; 18 : nat8; "OP"; null };
})' --ic


# frontend
dfx deploy frontend_prod --mode reinstall --ic