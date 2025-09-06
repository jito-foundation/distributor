#!/usr/bin/env bash
set -euo pipefail

: "${ANCHOR_PROVIDER_URL:?set ANCHOR_PROVIDER_URL}"
: "${ANCHOR_WALLET:?set ANCHOR_WALLET}"
: "${ANCHOR_PROVIDER_COMMITMENT:=finalized}"

# ---- per-repo knobs ----
PROG_SNAKE="merkle_distributor"   # reward_pool in the other repo
PROG_KEBAB="merkle-distributor"   # reward-pool in the other repo
# ------------------------

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Keep solana CLI aligned with Anchor URL
export SOLANA_URL="$ANCHOR_PROVIDER_URL"

mkdir -p target/deploy
[[ -f keys/${PROG_SNAKE}-keypair.json ]] && cp -f keys/${PROG_SNAKE}-keypair.json target/deploy/${PROG_SNAKE}-keypair.json

PROG_KEYPAIR="target/deploy/${PROG_SNAKE}-keypair.json"
SO="target/deploy/${PROG_SNAKE}.so"
IDL="target/idl/${PROG_SNAKE}.json"
PROG_ID="$(solana address -k "$PROG_KEYPAIR")"

echo "Stable program id: $PROG_ID"

# build only this package
anchor build -p "$PROG_SNAKE"

# deploy and capture signature (use same URL explicitly)
DEPLOY_JSON="$(solana -u "$ANCHOR_PROVIDER_URL" program deploy "$SO" --program-id "$PROG_KEYPAIR" --output json)"
SIG="$(echo "$DEPLOY_JSON" | awk -F'"' '/"signature":/ {print $4; exit}')"
echo "Deploy signature: ${SIG:-<none>}"
echo "Deploy success"

# confirm at finalized
if [[ -n "${SIG:-}" ]]; then
  solana -u "$ANCHOR_PROVIDER_URL" confirm -v "$SIG" --commitment "$ANCHOR_PROVIDER_COMMITMENT" >/dev/null
fi

# extra readiness loop: wait until loader says executable
for i in {1..40}; do
  if solana -u "$ANCHOR_PROVIDER_URL" program show "$PROG_ID" >/dev/null 2>&1; then
    # try a dry fetch of the IDL; if it errors with "account not found", that's fine
    if anchor idl fetch "$PROG_ID" >/dev/null 2>&1 || true; then
      break
    fi
  fi
  sleep 0.5
done

# print state we’re operating against (for debugging)
solana -u "$ANCHOR_PROVIDER_URL" program show "$PROG_ID"

# init/upgrade IDL (same RPC via env, with retries to smooth any last race)
try_idl() {
  if anchor idl fetch "$PROG_ID" >/dev/null 2>&1; then
    anchor idl upgrade "$PROG_ID" -f "$IDL"
  else
    anchor idl init "$PROG_ID" -f "$IDL"
  fi
}

for i in {1..6}; do
  if try_idl; then
    echo "IDL OK"
    exit 0
  fi
  echo "IDL attempt $i failed; retrying..."
  sleep 1
done

echo "ERROR: IDL init/upgrade failed after retries" >&2
exit 1
