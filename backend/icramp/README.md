## Generate the candid methods

In the root of the project:

```sh
cargo build --release --target wasm32-unknown-unknown --package backend
```

```sh
candid-extractor target/wasm32-unknown-unknown/release/backend.wasm > backend/backend.did
```

Generate declarations in the frontend canister

```sh
dfx generate
```

EVM Deployed in 0xBa84eF86624243b7AC5aee39beb259b1BDCc5F07

0x3f8e7De527263D8A059F87CA27E6143B373d3C7c

0x4316F5FC8fa58FbC6709E7A745f4FBb920Bf9C96

## Run tests

```sh
cargo test
```
