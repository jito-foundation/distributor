import { PublicKey } from "@solana/web3.js";
import { TOKEN_PROGRAM_ID } from "./env";
import { ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddressSync } from "@solana/spl-token";
import { MERKLE_DISTRIBUTOR_PROGRAM_ID } from "./env";

// Derives the MerkleDistributor PDA.
export function distributorPda(mint: PublicKey, version: bigint): PublicKey {
  const versionBuf = Buffer.alloc(8);
  versionBuf.writeBigUInt64LE(version);
  return PublicKey.findProgramAddressSync(
    [Buffer.from("MerkleDistributor"), mint.toBuffer(), versionBuf],
    MERKLE_DISTRIBUTOR_PROGRAM_ID,
  )[0];
}

// Derives the token vault ATA for the distributor over the given mint.
// Seeds in the Associated Token Program: [distributor, TOKEN_PROGRAM_ID, mint].
export function tokenVaultPda(distributor: PublicKey, mint: PublicKey): PublicKey {
  return getAssociatedTokenAddressSync(
    mint,
    distributor,
    true,
    TOKEN_PROGRAM_ID,
    ASSOCIATED_TOKEN_PROGRAM_ID,
  );
}

// Derives the ClaimStatus PDA for a claimant and distributor.
export function claimStatusPda(claimant: PublicKey, distributor: PublicKey): PublicKey {
  return PublicKey.findProgramAddressSync(
    [Buffer.from("ClaimStatus"), claimant.toBuffer(), distributor.toBuffer()],
    MERKLE_DISTRIBUTOR_PROGRAM_ID,
  )[0];
}
