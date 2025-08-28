# Solana Backend Canister

Solana canister for icRamp: address derivation, SOL/SPL transfers, a guarded token registry (allowlist), and a thin vault layer to mirror backend escrow state.

- **Escrow model**: users lock funds to the canister; backend coordinates vault entries + on-chain transfers.
- All token amounts are **base units** (lamports for SOL; raw mint units for SPL).

![Sequence: register_token → deposit_to_vault → lock_funds → send (create ATA if needed) → complete_order](/docs/images/solana-backend-vault-interaction.png)

## Features

- **Addresses**: derive canister & per-principal Solana pubkeys; compute ATAs.
- **Transfers**: send SOL and SPL (derives ATAs; you should ensure destination ATA exists or create it).
- **Token registry (allowlist)**: register SPL mints; decimals fetched on-chain; rejects unsupported mints.
- **Vaults**: stable maps for offramper/onramper balances (lamports + per-mint tokens).

## Setup and Deployment

### 0. Build + DID

```bash
cargo build --release --target wasm32-unknown-unknown --package solana_backend
candid-extractor target/wasm32-unknown-unknown/release/solana_backend.wasm > solana_backend/solana_backend.did
```

### 1. Deploy Solana RPC canister + API keys

```bash
dfx deploy sol_rpc

dfx canister call sol_rpc updateApiKeys "(vec {
  record { variant { AlchemyDevnet }; opt \"$ALCHEMY_KEY\" };
  record { variant { AnkrDevnet };   opt \"$ANKR_KEY\"   };
})"
```

### 2. Deploy `solana_backend`:

Example (Devnet + local ed25519 key):

```bash
dfx deploy solana_backend --argument "(
  variant { Reinstall = record {
    sol_rpc_canister_id = opt principal \"tghme-zyaaa-aaaar-qarca-cai\";
    ed25519_key_name    = variant { LocalDevelopment };
    network             = variant { Devnet };
    proxy_url           = \"https://ic2p2ramp.xyz\";
  }}
)"
```

Upgrade later (no state reset):

```bash
dfx deploy solana_backend --argument "(variant { Upgrade = null })" --upgrade-unchanged
```

## Quick API Cheatsheet (dfx)

### Addresses & Balances

```bash
# Canister’s Solana address
dfx canister call solana_backend canister_solana_account '()'

# SOL balance (lamports)
dfx canister call solana_backend get_balance '("<solana-pubkey>")'
```

### Token Registry (allowlist)

```bash
# Register by mint, symbol and rate symbol; decimals resolved on-chain
dfx canister call solana_backend register_tokens '(vec { record { "FxoGGtuyjfVybdA3X5WgxzNhjvSN73R5zqPYg3on8hwE"; "KONG"; "KONG" } })'

# Inspect registry
dfx canister call solana_backend get_registered_tokens '()'

# Check allowlist
dfx canister call solana_backend is_token_supported '("FxoGGtuyjfVybdA3X5WgxzNhjvSN73R5zqPYg3on8hwE")'
```

### Transfers

```bash
# Send SOL (lamports)
dfx canister call solana_backend send_sol '(opt principal "u6s2n-gx777-77774-qaaba-cai", "<to-pubkey>", 1000)'

# Send SPL (base units); ensure recipient ATA exists or create it first
dfx canister call solana_backend send_spl_token '(opt principal "u6s2n-gx777-77774-qaaba-cai", "FxoGGtuyjfVybdA3X5WgxzNhjvSN73R5zqPYg3on8hwE", "<to-pubkey>", 1000000)'
```

> Tip: 1 token with 6 decimals = `1_000_000` base units.

### Vaults

Two stable maps:

- `OFFRAMPER_VAULTS: Address -> VaultEntry`
- `ONRAMPER_VAULTS: Address -> VaultEntry`

`VaultEntry { lamports: u64, tokens: HashMap<mint, u64> }`

#### Read

```bash
dfx canister call solana_backend get_offramper_deposits '("<offramper-id>")'
dfx canister call solana_backend get_onramper_deposits  '("<onramper-id>")'
```

#### Flows

![icRamp Solana escrow flow — overview](/docs/images/solana-backend-diagram.png)

Using local identities as our offramper and onramper principals:

```bash
# create if not present
# dfx identity new maker
dfx identity use maker
export OFF=$(dfx identity get-principal)

# likewise for onramper
# dfx identity new taker
dfx identity use taker
export ON=$(dfx identity get-principal)
```

1. SOL: deposit → lock → complete

```bash
# deposit 1_000_000 lamports to OFF
dfx canister call solana_backend deposit_to_vault_canister '("'$OFF'", 1000000, null)'

# lock to ON
dfx canister call solana_backend lock_funds '("'$OFF'", "'$ON'", 1000000, null)'

# complete (release ON)
dfx canister call solana_backend complete_order '("'$ON'", 1000000, null)'
```

2. SPL: deposit → lock → unlock → cancel

```bash
MINT="FxoGGtuyjfVybdA3X5WgxzNhjvSN73R5zqPYg3on8hwE"

# deposit 1 token (6 decimals) to OFF
dfx canister call solana_backend deposit_to_vault_canister '("'$OFF'", 1000000, opt "'$MINT'")'

# lock to ON
dfx canister call solana_backend lock_funds '("'$OFF'", "'$ON'", 1000000, opt "'$MINT'")'

# unlock back to OFF
dfx canister call solana_backend unlock_funds '("'$OFF'", "'$ON'", 1000000, opt "'$MINT'")'

# cancel original OFF deposit
dfx canister call solana_backend cancel_deposit '("'$OFF'", 1000000, opt "'$MINT'")'
```

## Dev Notes

- **Public updates**: Methods you call via `dfx` must be `#[update] pub fn ...`. (E.g., `deposit_to_vault_canister`, `cancel_deposit`.)
- **Errors**, not panics: all RPC/parse errors bubble up via `Result`, never `panic!`.
- **Registry policy**: allowlist only by default. Decimals are read from the Mint (byte 44) and stored once.
- **Token-2022**: supported at owner check level; review per-mint extensions (fees/hooks) before enabling broadly.
- **Base units everywhere**: UI converts using stored decimals.

### Rebuild DID after changes

```bash
cargo build --release --target wasm32-unknown-unknown --package solana_backend
candid-extractor target/wasm32-unknown-unknown/release/solana_backend.wasm > solana_backend/solana_backend.did
dfx generate
```

### Troubleshooting

- `UnsupportedToken(...)`: mint not in allowlist → register_tokens first.
- `InvalidAccountData / simulation failed`: recipient ATA missing → create ATA before SPL transfer (or include a create-ATA instruction).
- `Source token account does not exist`: fund or create the sender’s ATA for that mint.
- `ParseError`: invalid base58 input (pubkey/mint).

---

## Contributing

Contributions are welcome! If you encounter any issues or have suggestions for improvements, please create an issue or submit a pull request.
