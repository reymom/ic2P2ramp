# SPL Token Utilities (Devnet)

Minimal helpers to:

1. Create a devnet SPL token (with initial supply)
2. Publish token metadata (Metaplex)
3. Mint & send more tokens to a Solflare wallet

> Uses `spl-token` + `solana` CLIs and `node` for metadata.

## Prereqs

- Solana CLI: https://docs.solana.com/cli/install-solana-cli-tools
- spl-token CLI: `cargo install spl-token-cli` or via Solana tools
- Node 18+
- A Solflare pubkey to receive tokens (`OWNER`)

## Setup

```bash
cd backend/solana/scripts/token_deploy
cp .env.example .env
# edit .env -> OWNER, optional: DECIMALS, INITIAL_SUPPLY, etc.
```

## 1) Create a token

```bash
source .env
./create_devnet_token.sh
# writes .mint and updates .env with MINT=...
```

## 2) Publish metadata

Prepare a public JSON (e.g. GitHub Gist raw URL) like:

```json
{
  "name": "BONK",
  "symbol": "BONK",
  "description": "Devnet BONK clone for local icRamp testing.",
  "image": "https://encrypted-tbn0.gstatic.com/images?q=tbn:ANd9GcQfxnrbtCKDBeUUpPI6gDeVAVvqYw1TkZMu-Us5BTuYQlhBEEsKgVnHDczXdJtQt4ulcXE&usqp=CAU",
  "extensions": { "website": "https://bonkcoin.com" }
}
```

Then:

```bash
source .env
node set_token_metadata.mjs
```

Env used by `set_token_metadata.mjs`:

- `RPC` (default devnet)
- `MINT` (from `.env` or manual)
- `NAME`, `SYMBOL`, `URI`
- `KEYPAIR` (payer/updateAuthority; default `~/.config/solana/id.json`)

## 3) Mint more & send to Solflare

```bash
source .env
# Mint + transfer 100000 tokens to OWNER
AMOUNT=100000 ./mint_and_send.sh
# Transfer only (no mint)
ONLY_TRANSFER=1 AMOUNT=25000 ./mint_and_send.sh
```

### Verify

```bash
spl-token supply "$MINT"
spl-token balance "$MINT" --owner "$OWNER"
spl-token mint-info "$MINT"
```

## Notes

- Amounts are human units (respect token `DECIMALS`).
- If you disabled the mint authority, you cannot mint more.
- `--fund-recipient` will auto-create the recipient ATA if missing.
