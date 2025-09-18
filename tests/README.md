# icRamp test workspace

This directory contains reproducible, deterministic integration tests for the icRamp backend canisters.  
We run everything inside **PocketIC**, with JSON-RPC **mocks** for Solana so no external calls are needed.

## Layout

```bash
├── bitcoin_tests
│   ├── Cargo.toml
│   └── src
│       ├── bitcoin_tests.rs
│       ├── env.rs
│       ├── helpers.rs
│       ├── lib.rs
│       ├── rune_tests.rs
│       ├── setup.rs
│       ├── token_tests.rs
│       └── vault_tests.rs
├── fixtures
│   └── wasm
│       ├── ic-btc-canister.wasm.gz
│       └── sol_rpc_canister.wasm.gz
├── README.md
├── solana_tests
│   ├── Cargo.toml
│   └── src
│       ├── env.rs
│       ├── helpers
│       │   ├── json.rs
│       │   ├── mock.rs
│       │   └── mod.rs
│       ├── lib.rs
│       ├── registry_tests.rs
│       ├── setup.rs
│       ├── solana_tests.rs
│       └── vault_tests.rs
└── testkit
    ├── Cargo.toml
    └── src
        ├── helpers.rs
        └── lib.rs
```

## Prereqs

- Rust stable
- `pocket-ic` binary (`/usr/local/bin/pocket-ic` by default)
- Build Solana backend wasm once:
  ```bash
  cargo build -p solana_backend --release --target wasm32-unknown-unknown
  ```

> You can also point tests to a custom path via `SOLANA_BACKEND_WASM=/abs/path/to/solana_backend.wasm`.

No internet/RPC keys required: Solana JSON-RPC is fully mocked.

## Running tests

### All Solana tests

```bash
cargo test -p solana_tests -- --nocapture
```

### Solana tests by module

1. Solana client operations

```bash
cargo test -p solana_tests solana_tests -- --nocapture
```

2. Registry flow

```bash
cargo test -p solana_tests registry_tests -- --nocapture
```

3. Vault flow

```bash
cargo test -p solana_tests vault_tests -- --nocapture
```

### A single Solana test

```bash
cargo test -p solana_tests registry_tests::test_register_and_query_success_token_2022 -- --nocapture
```

### All Bitcoin tests

```bash
cargo test -p bitcoin_tests -- --nocapture
```

## Solana mocking: how it works

- `pump_and_mock_http(pic, rounds, responder)` drains pending HTTP outcalls from PocketIC and feeds your responder’s JSON back in.
- `helpers/json.rs` only creates correctly-shaped JSON payloads (`resp_ctx_owner`, `resp_block`, `resp_token_balance`, …).
- `helpers/mock.rs` builds stateful responders that mirror Solana client flows:
  - `responder_mint_decimals_ok_token(decimals)`
  - `responder_mint_decimals_ok_token2022(decimals)`
  - `responder_wrong_owner()`
  - `responder_missing_mint_data_token()`
  - `responder_bad_b64_len_token(first, second)`
  - `responder_balance_zero_for_mint(mint)`
  - `responder_create_ata_flow(mint)`
  - `responder_send_sol()`
  - `responder_send_spl_token_create_dest_ata(mint)`

## Common patterns

- **Register a token** (RPC mocked once), then all vault ops run with no RPC:

```rust
// inside test
let msg = env.pic.submit_call(
    env.canister_id, Principal::anonymous(), "register_tokens",
    Encode!(&vec![(mint.clone(), "USDC".into(), "USDC".into())]).unwrap(),
).unwrap();

pump_and_mock_http(&env.pic, 20, responder_mint_decimals_ok_token(6));
let res: Result<()> = candid::decode_one(&env.pic.await_call(msg).unwrap()).unwrap();
res.expect("register_tokens should succeed");
```

- **Vault** (pure canister state): `deposit`, `cancel`, `lock`, `unlock`, `complete` — just pass `Some(mint)` for SPL, or `None` for SOL.

## Example result (Solana)

```bash
running 17 tests
...
test result: ok. 17 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```
