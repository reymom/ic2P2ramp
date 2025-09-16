#!/usr/bin/env bash
set -euo pipefail

# Required env:
#   OWNER               => Solflare owner pubkey (recipient)
# Optional env:
#   DECIMALS            => default 5
#   INITIAL_SUPPLY      => default 100000  (human units)
#   MINT_TO_SOLFLARE    => default 50000   (<= INITIAL_SUPPLY)
#   KEYPAIR             => default ~/.config/solana/id.json
#   RPC                 => default https://api.devnet.solana.com
#   WRITE_ENV           => "1" writes export MINT=... line to .env (default 1)

cd "$(dirname "$0")"

OWNER="${OWNER:?OWNER (Solflare pubkey) is required}"
DECIMALS="${DECIMALS:-5}"
INITIAL_SUPPLY="${INITIAL_SUPPLY:-100000}"
MINT_TO_SOLFLARE="${MINT_TO_SOLFLARE:-50000}"
KEYPAIR="${KEYPAIR:-$HOME/.config/solana/id.json}"
RPC="${RPC:-https://api.devnet.solana.com}"
WRITE_ENV="${WRITE_ENV:-1}"

command -v spl-token >/dev/null || { echo "spl-token CLI not found"; exit 1; }
command -v solana    >/dev/null || { echo "solana CLI not found"; exit 1; }

solana config set --url "$RPC" >/dev/null

# Ensure key exists (mint authority lives here)
[[ -f "$KEYPAIR" ]] || solana-keygen new -o "$KEYPAIR" --no-bip39-passphrase -s

# Pay fees
solana airdrop 2 >/dev/null 2>&1 || true

# Create mint with decimals
MINT=$(spl-token create-token --decimals "$DECIMALS" | awk '/Creating token/ {print $3}')
echo "Mint: $MINT" | tee .mint

# Ensure authority ATA
MY_ATA=$(spl-token create-account "$MINT" | awk '/Creating account/ {print $3}')
echo "My ATA: $MY_ATA"

# Mint initial supply to authority
spl-token mint "$MINT" "$INITIAL_SUPPLY"

# Transfer a chunk to Solflare (auto-create ATA)
spl-token transfer "$MINT" "$MINT_TO_SOLFLARE" "$OWNER" \
  --fund-recipient --allow-unfunded-recipient

echo
echo "=== Post-state ==="
spl-token accounts "$MINT" | sed 's/^/  /'
spl-token supply "$MINT"

# Export for later scripts
if [ "$WRITE_ENV" = "1" ]; then
  grep -q '^MINT=' .env 2>/dev/null && sed -i "s/^MINT=.*/MINT=$MINT/" .env || echo "MINT=$MINT" >> .env
fi

echo "Done. Devnet mint: $MINT"