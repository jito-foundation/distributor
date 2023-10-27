#!/usr/bin/env sh
# to use: ./s [HOSTNAME or IP]
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" &>/dev/null && pwd)"

HOST=$1

echo "Syncing to host: $HOST"

# sync + build
rsync -avh --delete --exclude target "$SCRIPT_DIR" "$HOST":~/
