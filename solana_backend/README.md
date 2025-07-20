# Solana Backend Canister

The Solana backend canister provides functionality for managing Solana transactions and supporting custom SPL‐token on the Internet Computer. This includes generating Solana addresses and creating instructions, transactions and registering SPL-token metadata. It serves as a foundational component for managing the Solana Vault in the icRamp platform.

## Features

## Vault Functionality

### Vault Overview

### Deposit Workflow

### Withdrawal Workflow

## Setup and Deployment

### 1. Generate the Candid File

Solana backend canister:

```bash
cargo build --release --target wasm32-unknown-unknown --package solana_backend
```

Extract the Candid methods from the compiled WebAssembly file:

```bash
candid-extractor target/wasm32-unknown-unknown/release/solana_backend.wasm > solana_backend/solana_backend.did
```

### 2. Generate TypeScript Declarations

To use the canister in your frontend application, generate TypeScript bindings:

```bash
dfx generate
```

This will output TypeScript declarations in the corresponding frontend canister directory.

### 3. Configuration

- **Solana Network**: Configure the canister to connect to `Testnet`, `Devnet`, or `Mainnet`. Update the `network` parameter in canister calls as required.
- **Vault Management**: Ensure the canister has sufficient lamports for operations.

## Using the Canister

### 1. Register SPL-tokens

Before handling tokens in your application, you must register their metadata with the Solana backend. This allows the canister to recognize and process specific SPL-tokens.

### 2. Fetch Solana and Token Balances

The backend canister supports fetching Solana and SPL-token balances.

### 3. Transfer Solana or SPL-tokens

- **Solana Transfer**

- **SPL-token Transfer**

## Development

### Address Generation

## Contributing

Contributions are welcome! If you encounter any issues or have suggestions for improvements, please create an issue or submit a pull request.
