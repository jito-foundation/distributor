# Merkle-distributor

A program and toolsets for distributing tokens efficiently via uploading a [Merkle root](https://en.wikipedia.org/wiki/Merkle_tree).

## Sharding merkle tree

Thanks Jito for excellent [Merkle-distributor project](https://github.com/jito-foundation/distributor). In Jupiter, We fork the project and add some extra steps to make it works for a large set of addresses. 

There are issues if the number of airdrop addresses increases:
- The size of the proof increases, that may be over solana transaction size limit.
- Too many write locked accounts duration hot claming event, so only some transactions are get through. 

In order to tackle it, we break the large set of addresses to smaller merkle trees, like 12000 addresses for each merkle tree. Therefore, when user claim, that would write lock on different accounts as well as reduces proof size. 

Before are follow toolset to build sharding merkle trees

## CLI
Build and deploy sharding merkle trees:

```
cd cli
cargo build
../target/debug/cli create-merkle-tree --csv-path [PATH_TO_LARGE_SET_OF_ADDRESS] --merkle-tree-path [PATH_TO_FOLDER_STORE_ALL_MERKLE_TREES] --max-nodes-per-tree 12000
../target/debug/cli --mint [TOKEN_MINT] --keypair-path [KEY_PAIR] --rpc-url [RPC] new-distributor --start-vesting-ts [START_VESTING] --end-vesting-ts [END_VESTING] --merkle-tree-path [PATH_TO_FOLDER_STORE_ALL_MERKLE_TREES] --clawback-start-ts [CLAWBACK_START] --enable-slot [ENABLE_SLOT]
../target/debug/cli --mint [TOKEN_MINT] --keypair-path [KEY_PAIR] --rpc-url [RPC] fund-all --merkle-tree-path [PATH_TO_FOLDER_STORE_ALL_MERKLE_TREES]
```

Anyone can verify the whole setup after that:

```
../target/debug/cli --mint [TOKEN_MINT] --keypair-path [KEY_PAIR] --rpc-url [RPC] verify --merkle-tree-path [PATH_TO_FOLDER_STORE_ALL_MERKLE_TREES] --clawback-start-ts [CLAWBACK_START] --enable-slot [ENABLE_SLOT] --admin [ADMIN]
```

## API
We can host API in local server 
```
cd api
cargo build
../target/debug/jupiter-airdrop-api --merkle-tree-path [PATH_TO_FOLDER_STORE_ALL_MERKLE_TREES] --rpc-url [RPC] --mint [TOKEN_MINT] --program-id [PROGRAM_ID]
```
