cargo build --release --target wasm32-unknown-unknown --package solana_backend

candid-extractor target/wasm32-unknown-unknown/release/solana_backend.wasm > solana_backend/solana_backend.did

dfx deploy sol_rpc

dfx canister call sol_rpc updateApiKeys "(vec {
  record { variant { AlchemyDevnet }; opt \"$ALCHEMY_KEY\" };
  record { variant { AnkrDevnet }; opt \"$ANKR_KEY\" };
})"

dfx deploy solana_backend --argument "(
    variant { 
        Reinstall = record {
            sol_rpc_canister_id = opt principal \"tghme-zyaaa-aaaar-qarca-cai\";
            ed25519_key_name = variant { LocalDevelopment };
            network = variant { Devnet }; 
            proxy_url = \"https://ic2p2ramp.xyz\";
        }
    }
)"

dfx deploy solana_backend --argument "( variant { Upgrade = null } )" --upgrade-unchanged

dfx deploy solana_backend --upgrade-unchanged --argument "(
    variant {
        Upgrade = opt record {
            network = null;
            sol_rpc_canister_id = opt principal \"tghme-zyaaa-aaaar-qarca-cai\";
            proxy_url = null;
        }
    }
)"