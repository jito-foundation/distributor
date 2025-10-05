# merkle-distributor

A program for distributing tokens efficiently via uploading a [Merkle root](https://en.wikipedia.org/wiki/Merkle_tree).

## 🚀 Recent Updates (2024-10-05)

This project has been successfully migrated to the latest Solana and Anchor versions for improved performance, security, and compatibility.

### ✅ **MAJOR FIXES COMPLETED**

**🔧 Critical Dependency Issues Resolved:**
- **Fixed `spl-pod` version conflict**: Resolved the `decode_error` and `PrintProgramError` issues that were preventing builds
- **Updated SPL dependencies**: `spl-associated-token-account` 3.0.0 → 6.0.0 for Solana 2.2.17 compatibility
- **Eliminated version conflicts**: All SPL dependencies now use consistent versions compatible with Solana 2.2.17

**📦 Migration Summary:**
- **Anchor Framework**: 0.28.0 → 0.31.1
- **Solana SDK**: 1.16.16 → 2.2.17 (Agave)
- **Rust Toolchain**: 1.72.0 → 1.90.0
- **SPL Token**: 2.2.0 → 6.0.0

**🔨 Key Changes:**
- ✅ Updated all dependencies to latest compatible versions
- ✅ Migrated from direct `solana_program` imports to `anchor_lang::solana_program`
- ✅ Fixed API changes in Anchor 0.31.1 (bumps.get() → bumps.distributor)
- ✅ Added `idl-build` feature for proper IDL generation
- ✅ Updated Rust to support 2024 edition dependencies
- ✅ **FIXED**: Resolved `spl-pod@0.2.3` vs `spl-pod@0.5.1` version conflicts
- ✅ **FIXED**: Eliminated `decode_error` and `PrintProgramError` compilation errors
- ✅ Successfully deployed to Solana devnet

**🚀 Deployment Details:**
- **Program ID**: `G9n7UVLKeGBx9kXJnutZdd8V92tHm2ssoLrzyX89btoD`
- **Network**: Devnet
- **Transaction**: `4dHeNc564ozTZEQuuv8jUybDkKvxMQR5nQZE6vMNewCF3fFVwmiis8fAi8NcuhXHZCYPcffKSa28pQXGPqe8Wc9t`

### Breaking Changes

1. **Import Changes**: All `solana_program` imports now use `anchor_lang::solana_program`
2. **Bumps API**: Changed from `ctx.bumps.get("account")` to `ctx.bumps.account`
3. **Dependencies**: Removed direct `solana_program` dependencies (now provided by Anchor)

### Development Environment

**Required Versions:**
- Rust: 1.90.0+
- Solana CLI: 2.2.17+
- Anchor CLI: 0.31.1+

**Installation:**
```bash
# Update Rust
rustup update stable

# Install Solana CLI
sh -c "$(curl -sSfL https://release.anza.xyz/stable/install)"

# Install Anchor CLI
cargo install --git https://github.com/coral-xyz/anchor avm --force
avm install 0.31.1
avm use 0.31.1
```

## Claiming Airdrop via CLI

To claim via CLI instead of using `https://jito.network/airdrop`, run the following commands.

1. Build the cli (must have rust + cargo installed):

```bash
cargo b -r
```

2. Run `claim` with the proper args. Be sure to replace `<YOUR KEYPAIR>` with the _full path_ of your keypair file. This will transfer tokens from the account `8Xm3tkQH581s3MoRHWUNYA5jKbgPATW4tJAAxgwDC6T6` to a the associated token account owned by your keypair, creating it if it doesn't exist.

```bash
./target/release/cli --rpc-url https://api.mainnet-beta.solana.com --keypair-path <YOUR KEYPAIR> --airdrop-version 0 --mint jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL --program-id mERKcfxMC5SqJn4Ld4BUris3WKZZ1ojjWJ3A3J5CKxv claim --merkle-tree-path merkle_tree.json
```

Note that for searchers and validators, not all tokens will be vested until December 7, 2024. You can check the vesting status at `https://jito.network/airdrop`.

## 🛠️ **Complete CLI Usage Guide**

### **Build All Components**
```bash
# Build everything (program, CLI, API)
cargo build --release

# Or build just the CLI
cargo build --release --bin cli

# Or build just the API
cargo build --release --bin jito-airdrop-api
```

### **CLI Commands Overview**

The CLI tool (`./target/release/cli`) provides comprehensive functionality for managing merkle distributors:

#### **1. Create Merkle Tree**
```bash
./target/release/cli \
  --rpc-url https://api.devnet.solana.com \
  --keypair-path /path/to/your/keypair.json \
  --airdrop-version 0 \
  --mint <TOKEN_MINT_ADDRESS> \
  --program-id <PROGRAM_ID> \
  create-merkle-tree \
  --csv-path recipients.csv \
  --merkle-tree-path merkle_tree.json
```

**CSV Format Required:**
```csv
pubkey,amount_unlocked,amount_locked,category
11111111111111111111111111111112,1000000,2000000,Staker
22222222222222222222222222222223,2000000,4000000,Validator
33333333333333333333333333333334,1500000,3000000,Searcher
```

#### **2. Create New Distributor**
```bash
./target/release/cli \
  --rpc-url https://api.devnet.solana.com \
  --keypair-path /path/to/your/keypair.json \
  --airdrop-version 0 \
  --mint <TOKEN_MINT_ADDRESS> \
  --program-id <PROGRAM_ID> \
  new-distributor \
  --clawback-receiver-token-account <TOKEN_ACCOUNT> \
  --start-vesting-ts $(date +%s) \
  --end-vesting-ts $(($(date +%s) + 86400)) \
  --clawback-start-ts $(($(date +%s) + 172800)) \
  --merkle-tree-path merkle_tree.json
```

#### **3. Claim Tokens**
```bash
./target/release/cli \
  --rpc-url https://api.devnet.solana.com \
  --keypair-path /path/to/your/keypair.json \
  --airdrop-version 0 \
  --mint <TOKEN_MINT_ADDRESS> \
  --program-id <PROGRAM_ID> \
  claim \
  --merkle-tree-path merkle_tree.json
```

#### **4. Set Admin**
```bash
./target/release/cli \
  --rpc-url https://api.devnet.solana.com \
  --keypair-path /path/to/your/keypair.json \
  --airdrop-version 0 \
  --mint <TOKEN_MINT_ADDRESS> \
  --program-id <PROGRAM_ID> \
  set-admin \
  --new-admin <NEW_ADMIN_PUBKEY>
```

### **API Server Usage**

#### **Start the API Server**
```bash
./target/release/jito-airdrop-api \
  --bind-addr 0.0.0.0:7001 \
  --merkle-tree-path merkle_tree.json \
  --rpc-url https://api.devnet.solana.com \
  --mint <TOKEN_MINT_ADDRESS> \
  --program-id <PROGRAM_ID> \
  --airdrop-version 0 \
  --enable-proof-endpoint
```

#### **API Endpoints**
- `GET /` - Root endpoint
- `GET /version` - Get airdrop version
- `GET /users` - List all eligible users
- `GET /distributor` - Get distributor information
- `GET /status/{user_pubkey}` - Get user claim status
- `GET /proof/{user_pubkey}` - Get user merkle proof (requires `--enable-proof-endpoint`)

### **Complete Workflow Example**

```bash
# 1. Build everything
cargo build --release

# 2. Deploy the program
anchor deploy

# 3. Create recipients CSV
echo "pubkey,amount_unlocked,amount_locked,category" > recipients.csv
echo "11111111111111111111111111111112,1000000,2000000,Staker" >> recipients.csv

# 4. Create merkle tree
./target/release/cli \
  --rpc-url https://api.devnet.solana.com \
  --keypair-path /path/to/keypair.json \
  --airdrop-version 0 \
  --mint jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL \
  --program-id G9n7UVLKeGBx9kXJnutZdd8V92tHm2ssoLrzyX89btoD \
  create-merkle-tree \
  --csv-path recipients.csv \
  --merkle-tree-path merkle_tree.json

# 5. Create distributor (you'll need a token account)
./target/release/cli \
  --rpc-url https://api.devnet.solana.com \
  --keypair-path /path/to/keypair.json \
  --airdrop-version 0 \
  --mint jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL \
  --program-id G9n7UVLKeGBx9kXJnutZdd8V92tHm2ssoLrzyX89btoD \
  new-distributor \
  --clawback-receiver-token-account <TOKEN_ACCOUNT> \
  --start-vesting-ts $(date +%s) \
  --end-vesting-ts $(($(date +%s) + 86400)) \
  --clawback-start-ts $(($(date +%s) + 172800)) \
  --merkle-tree-path merkle_tree.json

# 6. Start API server
./target/release/jito-airdrop-api \
  --bind-addr 0.0.0.0:7001 \
  --merkle-tree-path merkle_tree.json \
  --rpc-url https://api.devnet.solana.com \
  --mint jtojtomepa8beP8AuQc6eXt5FriJwfFMwQx2v2f9mCL \
  --program-id G9n7UVLKeGBx9kXJnutZdd8V92tHm2ssoLrzyX89btoD \
  --airdrop-version 0 \
  --enable-proof-endpoint

# 7. Test API endpoints
curl http://localhost:7001/
curl http://localhost:7001/users
curl http://localhost:7001/status/11111111111111111111111111111112
```

## Building and Deploying

### Build the Program
```bash
anchor build
```

### Deploy to Devnet
```bash
anchor deploy
```

### Deploy to Mainnet
```bash
# Switch to mainnet
solana config set --url https://api.mainnet-beta.solana.com

# Update Anchor.toml cluster to "mainnet"
# Update Anchor.toml wallet to your mainnet keypair

# Deploy
anchor deploy
```


## Key Features

- **Vesting System**: Implements time-locked token distribution
- **Merkle Proof Verification**: Secure, gas-efficient claiming
- **Clawback Mechanism**: Admin can reclaim unclaimed tokens
- **Frontrunning Protection**: Built-in security measures
- **Multi-Component Architecture**: Program, API, CLI, and utilities

## 🐛 **Troubleshooting**

### **Common Issues & Solutions**

#### **1. Build Errors**
```bash
# If you get spl-pod version conflicts:
error[E0433]: failed to resolve: could not find `decode_error` in `solana_program`
error[E0405]: cannot find trait `PrintProgramError` in module `solana_program::program_error`
```
**Solution**: This is fixed! We resolved the `spl-pod@0.2.3` vs `spl-pod@0.5.1` conflict by updating `spl-associated-token-account` to version 6.0.0.

#### **2. Rust Edition Errors**
```bash
# If you get:
this version of Cargo is older than the `2024` edition
```
**Solution**: Update Rust toolchain:
```bash
rustup update stable
rustup default stable
```

#### **3. Dependency Conflicts**
```bash
# If you get version conflicts:
error: failed to select a version for the requirement `solana-sdk = "^2.2.17"`
```
**Solution**: Clean and rebuild:
```bash
cargo clean
rm -f Cargo.lock
cargo update
cargo build --release
```

#### **4. API Server Issues**
```bash
# If API fails to start:
Error: MerkleDistributorError("Merkle Distributor not found")
```
**Solution**: You need to create a distributor account first:
```bash
# Create merkle tree
./target/release/cli ... create-merkle-tree ...

# Create distributor account
./target/release/cli ... new-distributor ...

# Then start API
./target/release/jito-airdrop-api ...
```

#### **5. SPL Token Account Creation Issues**
```bash
# If you get:
Error: IncorrectProgramId
```
**Solution**: This is a known issue with SPL token program compatibility. Use the CLI to create accounts or use a different approach for token account creation.

#### **6. Deployment Issues**
```bash
# If deployment fails:
Error: Dynamic program error: No default signer found
```
**Solution**: Configure Solana CLI:
```bash
solana config set --keypair /path/to/your/keypair.json --url devnet
```

### **Verification Steps**

1. **Check Build Success**:
   ```bash
   cargo build --release
   # Should complete without errors
   ```

2. **Verify Program Deployment**:
   ```bash
   anchor deploy
   # Should show "Deploy success" with Program ID
   ```

3. **Test CLI Functionality**:
   ```bash
   ./target/release/cli --help
   # Should show all available commands
   ```

4. **Test API Endpoints**:
   ```bash
   curl http://localhost:7001/
   # Should return "Jito Airdrop API"
   ```

### **Getting Help**

If you encounter issues not covered here:

1. **Check the logs**: Look for specific error messages in the terminal output
2. **Verify versions**: Ensure all tools match the required versions
3. **Clean rebuild**: Try `cargo clean && cargo build --release`
4. **Check network**: Ensure you're connected to the correct Solana network (devnet/mainnet)
