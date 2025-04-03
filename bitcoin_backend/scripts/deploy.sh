cargo build --release --target wasm32-unknown-unknown --package bitcoin_backend

candid-extractor target/wasm32-unknown-unknown/release/bitcoin_backend.wasm > bitcoin_backend/bitcoin_backend.did

dfx deploy bitcoin_backend --specified-id zhuzm-wqaaa-aaaap-qpk2q-cai --argument "(
    variant { 
        Reinstall = record { 
            network = variant { tegtest }; 
            proxy_url = \"https://ic2p2ramp.xyz\";
            unisat = record {
                api_url = \"open-api-testnet4.unisat.io\";
                api_key = \"${UNISAT_API_KEY}\";
            }; 
        }
    }
)"

dfx canister create --with-cycles 1_000_000_000_000 bitcoin_backend --ic

dfx deploy bitcoin_backend --argument "(
    variant { 
        Reinstall = record { 
            network = variant { testnet }; 
            proxy_url = \"https://ic2p2ramp.xyz\";
            unisat = record {
                api_url = \"open-api-testnet4.unisat.io\";
                api_key = \"${UNISAT_API_KEY}\";
            }; 
        }
    }
)" --ic
