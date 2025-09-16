#!/usr/bin/env bash
set -euo pipefail

# Required env:
#   OWNER               => destination owner (Solflare) pubkey (e.g. CnzU...SWsUj)
# Optional env:
#   AMOUNT              => tokens (human units). If omitted, must pass as $1
#   MINT                => mint address; if missing, read from .mint
#   KEYPAIR             => path to payer/authority key (default: ~/.config/solana/id.json)
#   RPC                 => default https://api.devnet.solana.com
#   FUND_RECIPIENT      => "1" to auto-create ATA for recipient (default: 1)
#   ONLY_TRANSFER       => "1" to skip minting and only transfer (default: 0)

cd "$(dirname "$0")"

AMOUNT="${AMOUNT:-${1:-}}"
[[ -n "${AMOUNT}" ]] || { echo "AMOUNT not set (env or arg 1)"; exit 1; }

MINT="${MINT:-$( [ -f .mint ] && cat .mint || true )}"
[[ -n "${MINT}" ]] || { echo "MINT not set and .mint not found"; exit 1; }

OWNER="${OWNER:?OWNER (Solflare pubkey) is required}"
KEYPAIR="${KEYPAIR:-$HOME/.config/solana/id.json}"
RPC="${RPC:-https://api.devnet.solana.com}"
FUND_RECIPIENT="${FUND_RECIPIENT:-1}"
ONLY_TRANSFER="${ONLY_TRANSFER:-0}"

command -v spl-token >/dev/null || { echo "spl-token CLI not found"; exit 1; }
command -v solana    >/dev/null || { echo "solana CLI not found"; exit 1; }

solana config set --url "$RPC" >/dev/null

# fees (ignore failures)
solana airdrop 1 >/dev/null 2>&1 || true

if [ "$ONLY_TRANSFER" != "1" ]; then
  echo "Minting ${AMOUNT} tokens to mint-authority ATA…"
  spl-token mint "$MINT" "$AMOUNT" --owner "$KEYPAIR"
  echo
fi

echo "Transferring ${AMOUNT} tokens to $OWNER…"
if [ "$FUND_RECIPIENT" = "1" ]; then
  spl-token transfer "$MINT" "$AMOUNT" "$OWNER" \
    --owner "$KEYPAIR" --fee-payer "$KEYPAIR" \
    --fund-recipient --allow-unfunded-recipient
else
  spl-token transfer "$MINT" "$AMOUNT" "$OWNER" \
    --owner "$KEYPAIR" --fee-payer "$KEYPAIR"
fi

echo
echo "=== Verify ==="
spl-token supply "$MINT"
echo
spl-token balance "$MINT" --owner "$OWNER" || true