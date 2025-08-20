# merkle-distributor

A program for distributing tokens efficiently via uploading a [Merkle root](https://en.wikipedia.org/wiki/Merkle_tree).

## Local Dev Quickstart (macOS & Ubuntu)

```bash
rustup default 1.83.0
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"
export PATH="$HOME/.local/share/solana/install/active_release/bin:$PATH"
cargo install --git https://github.com/coral-xyz/anchor avm --force
avm install 0.31.1 && avm use 0.31.1
brew install node
npm i -g yarn

pkill -f solana-test-validator 2>/dev/null || true
solana-test-validator -r --quiet & sleep 5
solana config set -ul
solana airdrop 2

anchor build -p merkle_distributor
anchor test  -p merkle_distributor
```
