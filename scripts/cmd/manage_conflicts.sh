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

dfx canister call backend create_bitcoin_order_with_tx '(
    "3a30c17104a45214bd926a8dbde30c6af892db99d3c41bb5f399d781796af2cc",
    "tb1p8gskffp4t4amg6kn7vcs7w5qh0wdq7zcw736lt9zctqday3vfa9q7vwhpr",
    1 : nat64,
    "tb1prmkx3hvhttcga8z0n28jalzca0wemn8fp5gaj5lncw6cy4lcrnnszpve2m",
    vec {
        record { 
            variant { PayPal }; 
            variant { PayPal = record { id = "sb-ioze230588840@personal.example.com" : text } }; 
        }
    },
    "USD",
    15 : nat,
    opt "73393:191"
)'

dfx canister call backend retry_order_completion '(1: nat64)' --ic
