dfx canister call backend create_evm_order_with_tx '(
    84532 : nat64,
    "0x94b2397eb80a6515ece027efe296b05a78f134446951aa2ea15d13baeeeffb4c",
    1 : nat64,
    "0x632b39E5Fe4EAAFDF21601b2Bc206ca0f602C85A",
    vec {
        record { 
            variant { PayPal }; 
            variant { PayPal = record { id = "sb-ioze230588840@personal.example.com" : text } }; 
        }
    },
    "USD",
    10000000 : nat,
    opt "0x036CbD53842c5426634e7929541eC2318f3dCF7e"
)'

dfx canister call backend test_estimate_gas_commit '(
    84532, 
    "0x632b39E5Fe4EAAFDF21601b2Bc206ca0f602C85A",
    null,
    10000000000000000
)'