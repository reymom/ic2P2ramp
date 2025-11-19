# icramp_backend — Core Business Logic Canister

This canister implements the full business logic of icRamp:

- Orders & fills
- Liquid Orders (partial fills + top-ups)
- Payment providers (PayPal, Revolut, Stripe, Crypto)
- Multi-chain vault orchestration (BTC/EVM/SOL/ICP)
- Stripe & PayPal verification via HTTPS Outcalls
- Pay-With-Crypto on-chain verification

### 🔗 Related Documentation

See full platform documentation in the root README:  
`../README.md`

### 📁 Code Structure

- `src/` — order management, vault logic, listeners
- `model/` — payment providers, types, errors
- `evm/` — calls to the EVM vault
- `ipc/` — interaction with ICP
- `management/` — vault, token registry, orders management
- `inter_canister/` — interaction with bitcoin and solana canisters
- `outcalls/` — all the interactions with external APIs
