set -euo pipefail

SOLFLARE_OWNER="${OWNER}"
DECIMALS="${DECIMALS:-5}"           # BONK has 5 decimals
INITIAL_SUPPLY="${INITIAL_SUPPLY:-100000}"  # human units
MINT_TO_SOLFLARE="${MINT_TO_SOLFLARE:-50000}" # amount to move to Solflare (<= INITIAL_SUPPLY)

solana config set --url https://api.devnet.solana.com >/dev/null

# Keep your existing key; only create if missing (mint authority lives here)
[[ -f ~/.config/solana/id.json ]] || solana-keygen new -o ~/.config/solana/id.json --no-bip39-passphrase -s

# Make sure you can pay fees
solana airdrop 2 || true

# Create mint with desired decimals
MINT=$(spl-token create-token --decimals "$DECIMALS" | awk '/Creating token/ {print $3}')
echo "Mint: $MINT"

# Ensure YOUR ATA exists (mint authority’s owner)
MY_ATA=$(spl-token create-account "$MINT" | awk '/Creating account/ {print $3}')
echo "My ATA: $MY_ATA"

# Mint initial supply to YOUR ATA
spl-token mint "$MINT" "$INITIAL_SUPPLY"

# Transfer to Solflare, auto-creating their ATA and paying for it
spl-token transfer "$MINT" "$MINT_TO_SOLFLARE" "$SOLFLARE_OWNER" --fund-recipient --allow-unfunded-recipient

# Show balances
echo "=== Post-state ==="
spl-token accounts "$MINT" | sed 's/^/  /'
spl-token supply "$MINT"
echo "Done. Devnet mint: $MINT"
