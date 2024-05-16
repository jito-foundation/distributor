# Solana Governance token distribute

This repository is forked from Jito's [Merkle Distributor](https://github.com/jito-foundation/distributor) and intended to be used for the distribution of vote-specific tokens for validator governance.

# Voting process

1. After a proposal completes its period on the governance forum a stake weight capture & verification epoch begins
2. Using the [SPL Feature Proposal CLI](https://spl.solana.com/feature-proposal) a CSV file with current stake weights is generated
3. A merkle tree is generated using this CLI and the CSV file and both are uploaded here in a branch named after the SIMD
4. The voting token mint address, CSV file, CSV hash and merkle tree hash are posted to the forum proposal post
5. In the subsequent epoch the tokens are minted to the distributor vault and the mint authority is burned, validators can claim their tokens and vote by transferring them to a designated account (also posted in the forum proposal)

# Claiming voting tokens via CLI

You can either use the CLI from the Jito repository above if you feel more comfortable, or from this repository.

The changes in this repository to the CLI: 

- Adapt to the CSV format created from the SPL Feature Proposal CLI with validator stake weights, including removing UI decimal conversion
- Remove the unlocked token claim as there are no unlocked tokens and this creates a CLI error

**If you use Jito's CLI tool when you claim your tokens you will get a Rust panic/error, as it tries to claim both unlocked and locked tokens, however there won't be any locked tokens. Ignore this and check your token balance with `spl-token accounts --owner <YOUR_IDENTITY>`**

To claim via CLI

1. Build the cli (must have rust (min 1.68.0) + cargo installed):

```bash
cargo b -r
```

2. Run `claim` with the proper args. Be sure to replace `<YOUR KEYPAIR>` with the _full path_ of your identity keypair file. This will transfer tokens to a the associated token account owned by your keypair, creating it if it doesn't exist.

```bash
./target/release/cli --rpc-url https://api.mainnet-beta.solana.com --keypair-path <YOUR KEYPAIR> --airdrop-version 0 --mint <VOTE_MINT> --program-id mERKcfxMC5SqJn4Ld4BUris3WKZZ1ojjWJ3A3J5CKxv claim --merkle-tree-path ./votes/<SIMD>/merkle_tree.json
```

## Troubleshooting
If you have the incorrect version of Rust you can use `rustup` to set the default toolchain `rustup defautl 1.68.0`

