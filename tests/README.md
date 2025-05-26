```bash
cargo test --package bitcoin_backend_tests --lib -- integration::bitcoin_tests --show-output
```

```bash
$ cargo test --package bitcoin_backend_tests integration::bitcoin_tests::test_get -- --nocapture

[INFO] PocketIC server listening on port 35991
[Canister tqzl2-...] [init]: state = State { proxy_url: "https://example.xyz", ... }

p2tr_raw_key_spend_address = "bcrt1pgle5qc05pgnnyngrctcr0nkvvl60ylcuu247fmr68jlkpzad2p4q769ygy"
p2pkh_address = "mgW7FXrgzMV3QkP45Qa8STcLGnkHkWQPHv"
p2tr_script_spend_address = "bcrt1p4hpknmastamuawkxzhz2zr2ra050j6u8veywxqh9q5kda2yf7e5sa9ecsu"

test integration::bitcoin_tests::test_get_p2tr_raw_key_spend_address_balance ... ok
test integration::bitcoin_tests::test_get_p2pkh_address_balance ... ok
test integration::bitcoin_tests::test_get_p2tr_script_spend_inscription ... ok
test integration::bitcoin_tests::test_get_p2tr_script_spend_rune_etching ... ok

test result: ok. 4 passed; finished in 3.03s
```

```bash
$ cargo test --package bitcoin_backend_tests integration::vault_tests::test_btc_vault -- --nocapture

[INFO] PocketIC server listening on port 45227
[Canister tqzl2-...] [init]: state = State { proxy_url: "https://example.xyz", ... }
[Canister g4xu7-...] Starting heartbeat...
[Canister g4xu7-...] Sending GetSuccessorsRequestInitial { network: Regtest, ... }
Deployed canister ID: tqzl2-...
test integration::vault_tests::test_btc_vault ... ok

test result: ok. 1 passed; finished in 2.88s
```
