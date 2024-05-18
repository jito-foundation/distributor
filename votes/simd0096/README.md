# SIMD-0096 Reward full priority fee to validator

The SIMD can be viewed [on Github](https://github.com/solana-foundation/solana-improvement-documents/blob/main/proposals/0096-reward-collected-priority-fee-in-entirety.md) and a governance [forum proposal](https://forum.solana.com/t/proposal-for-enabling-the-reward-full-priority-fee-to-validator-on-solana-mainnet-beta/1456) has been posted.

The stake weight gathering process takes place in epoch 616 and the voting process will begin in epoch 617 and last until epoch 620.

The token distribution occurs via merkle distributor (see this repo). Validators need to claim their voting tokens using their identity account.

## Key addresses and hashes

### File hashes

Verify the `feature_proposal.csv` file has a hash of `3972a374683e9fe9bf63ca179ae603ebf26e9cd5`

```bash
cat feature-proposal.csv | sort | shasum
```

Verify the merkle tree has a hash of `3678a96f40de4b6eec8eedbc26c9b731f502362b`

```bash
cat simd-0096-merkle-tree.json | shasum
```

The vote token mint address is `simd96Cuw3M5TYAkZ1d71ug4bvVHiqHhhJzsFHHQxgq`

The total supply will be `368296676892441006` with 1802 participating validators.

To reproduce the merkle tree:

```bash
./target/release/cli --keypair-path /any/funded/keypair.json --rpc-url https://api.mainnet-beta.solana.com --mint simd96Cuw3M5TYAkZ1d71ug4bvVHiqHhhJzsFHHQxgq create-merkle-tree --csv-path ./votes/simd0096/feature-proposal.csv --merkle-tree-path simd-0096-merkle-tree-to-verify.json
```

This will generate a merkle tree which you can then compare against the one published here.

## Voting

Cast your vote with `spl-token transfer` to one of the following addresses:

**YES**
```bash
YESsimd96Cuw3M5TYAkZ1d71ug4bvVHiqHhhJzsFHHQ
```

**NO**
```bash
nosimd96Cuw3M5TYAkZ1d71ug4bvVHiqHhhJzssFHHQ
```

**ABSTAIN**
```bash
ABSTA1Nsimd96Cuw3M5TYAkZ1d71ug4bvVHiqHhhJzs
```

`spl-token transfer simd96Cuw3M5TYAkZ1d71ug4bvVHiqHhhJzsFHHQxgq ALL <VOTE_ADDRESS>`

## Tally / Results
Use the tally.sh script in this directory to check the results at any time.

`bash tally.sh`
