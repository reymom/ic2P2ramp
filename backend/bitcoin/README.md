# Bitcoin Backend Canister

The Bitcoin backend canister provides functionality for managing Bitcoin transactions and supporting custom Runes on the Internet Computer. This includes generating Bitcoin addresses, transferring Bitcoin or Runes, and registering Rune metadata. It serves as a foundational component for managing the Bitcoin Vault in the icRamp platform.

## Features

1. **Bitcoin Vault**:
   - Acts as a secure holding space for onramped Bitcoin until the order is processed.
   - Supports deposit, withdrawal, and balance query operations.
   - Provides secure handling of UTXOs for Bitcoin payments and script-based transactions.
2. **Rune Integration**:
   - Supports registering Runes (Bitcoin-based tokens) with metadata.
   - Handles Rune balances, transfers, and queries.
3. **Bitcoin Address Management**:
   - Generates addresses for P2PKH (legacy), P2SH (segregated witness), and P2TR (Taproot) transactions.
4. **Transaction Management**:
   - Facilitates Bitcoin transfers between users, canisters, and external addresses.
   - Allows the sending of Runes with custom OP_RETURN data.

## Vault Functionality

### Vault Overview

The Bitcoin Vault in the backend canister securely holds Bitcoin and Runes for users. It integrates with the onramping platform to:

- Receive Bitcoin deposits from users.
- Track balances per user and update them upon deposit confirmation.
- Enable withdrawal requests upon order completion.

### Deposit Workflow

1.  **Generate Bitcoin Vault Address**:
    Use the backend canister to generate a secure address for depositing Bitcoin into the vault:

```bash
dfx canister call bitcoin_fusion get_p2tr_script_spend_address
```

2.  **Confirm Deposits**:
    Once Bitcoin is deposited, the vault monitors the UTXOs using the Bitcoin network and confirms the balance:

```bash
dfx canister call bitcoin_fusion get_vault_balance '(principal "<USER_PRINCIPAL>")'
```

3.  **Lock Funds for Order**:
    When a user creates an order, the required amount of Bitcoin is locked in the vault:

```bash
dfx canister call bitcoin_fusion lock_vault_funds '(
    principal "<USER_PRINCIPAL>",
    100_000 : nat64  // Amount in satoshis
)'
```

### Withdrawal Workflow

1.  **Create Withdrawal Request**:
    Upon order completion, funds are released from the vault and sent to the recipient's address:

```bash
dfx canister call bitcoin_fusion release_vault_funds '(
    principal "<USER_PRINCIPAL>",
    "tb1qxyz...": text,  // Destination address
    100_000 : nat64      // Amount in satoshis
)'
```

2.  **Monitor Transactions**:
    The canister monitors the Bitcoin network to ensure withdrawals are confirmed.

## Setup and Deployment

### 1. Generate the Candid File

To generate the Candid interface for the Bitcoin backend canister:

```bash
cargo build --release --target wasm32-unknown-unknown --package bitcoin
```

Extract the Candid methods from the compiled WebAssembly file:

```bash
candid-extractor target/wasm32-unknown-unknown/release/bitcoin.wasm > bitcoin/bitcoin.did
```

### 2. Generate TypeScript Declarations

To use the canister in your frontend application, generate TypeScript bindings:

```bash
dfx generate
```

This will output TypeScript declarations in the corresponding frontend canister directory.

### 3. Configuration

- **Bitcoin Network**: Configure the canister to connect to `Testnet`, `Regtest`, or `Mainnet`. Update the `network` parameter in canister calls as required.
- **Vault UTXO Management**: Ensure the canister has access to sufficient UTXOs for operations.

## Using the Canister

### 1. Register Runes

Before handling Runes in your application, you must register their metadata with the Bitcoin backend. This allows the canister to recognize and process specific Runes.

### Example Command:

```bash
dfx canister call bitcoin_fusion register_runes '( vec {
    record { "🐕"; 5 : nat8; 100_000_000_000 : nat64; 100_000_000_000 : nat64 }; // DOG•GO•TO•THE•MOON
    record { "🤖"; 5 : nat8; 1_000_000_000 : nat64; 1_000_000_000 : nat64 };     // ARTIFICIAL•PUPPET
    record { "🐸"; 5 : nat8; 20_814_270_000 : nat64; 20_790_000_000 : nat64 };   // BITCOIN•FROGS
    record { "🧙"; 0 : nat8; 10_000 : nat64; 10_000 : nat64 };                   // MAKE•BITCOIN•MAGICAL•AGAIN
})'
```

Each record includes:

- **Symbol**: The emoji or identifier of the Rune (e.g., `"🐕"`).
- **Divisibility**: The number of decimals (e.g., `5`).
- **Cap**: The maximum supply of the Rune (e.g., `100_000_000_000`).
- **Premine**: The pre-mined supply (e.g., `100_000_000_000`).

### Note:

Runes must be registered before their balances or transfers can be processed.

### 2. Fetch Bitcoin and Rune Balances

The backend canister supports fetching Bitcoin and Rune balances. To fetch the balance of a specific Rune:

```bash
dfx canister call bitcoin_fusion get_rune_balance '(principal "<USER_PRINCIPAL>", "🐕")'
```

### 3. Transfer Bitcoin or Runes

To transfer Bitcoin or Runes, use the following methods:

- **Bitcoin Transfer**:

```bash
dfx canister call bitcoin_fusion send_btc_or_rune '(
    record {
        network = variant { Testnet };
        derivation_path = vec {};
        key_name = "my-key";
        dst_address = "tb1...";
        amount = 1_000_000 : nat64;
        rune = null;
        use_taproot = true
    }
)'
```

- **Rune Transfer**:

```bash
dfx canister call bitcoin_fusion send_btc_or_rune '(
    record {
        network = variant { Testnet };
        derivation_path = vec {};
        key_name = "my-key";
        dst_address = "tb1...";
        amount = 10_000 : nat64;
        rune = opt record { symbol = "🐕" };
        use_taproot = true
    }
)'
```

## Development

### Address Generation

The canister supports generating Bitcoin addresses for receiving funds. Example functions:

- **P2PKH (Legacy)**: `get_p2pkh_address`
- **P2TR (Taproot)**: `get_p2tr_script_spend_address`

### Testing with Local Bitcoin Regtest

To test functionality locally:

1. Set up a Bitcoin regtest node.
2. Configure the canister to use the `Regtest` network.
3. Use tools like `bitcoin-cli` to simulate transactions and UTXO generation.

## Key Notes

- The canister currently supports Bitcoin Testnet and Regtest networks for testing purposes. Mainnet support requires proper key management and testing.
- **Taproot Support**: The canister uses Taproot script-spend functionality for advanced transaction workflows.
- **Security**: Ensure keys and derivation paths are securely managed in production environments.

## Contributing

Contributions are welcome! If you encounter any issues or have suggestions for improvements, please create an issue or submit a pull request.
