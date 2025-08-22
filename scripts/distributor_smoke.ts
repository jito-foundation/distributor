import * as anchor from "@coral-xyz/anchor";
import { Program, BN, web3 } from "@coral-xyz/anchor";
import { Token, TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID, getAssociatedTokenAddress } from "@solana/spl-token";
import { keccak_256 } from "js-sha3";
import idl from "../app/idl/merkle_distributor.json" assert { type: "json" };

// Helper hashing utilities
function keccak(...buffers: Buffer[]): Buffer {
  const hash = keccak_256.create();
  buffers.forEach((b) => hash.update(b));
  return Buffer.from(hash.digest());
}

function hashLeaf(claimant: web3.PublicKey, unlocked: BN, locked: BN): Buffer {
  return keccak(
    Buffer.from([0]),
    claimant.toBuffer(),
    unlocked.toArrayLike(Buffer, "le", 8),
    locked.toArrayLike(Buffer, "le", 8)
  );
}

function hashNode(a: Buffer, b: Buffer): Buffer {
  const [l, r] = Buffer.compare(a, b) <= 0 ? [a, b] : [b, a];
  return keccak(Buffer.from([1]), l, r);
}

(async () => {
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);
  const connection = provider.connection;
  const admin = provider.wallet.payer as web3.Keypair;
  const programId = new web3.PublicKey("mERKcfxMC5SqJn4Ld4BUris3WKZZ1ojjWJ3A3J5CKxv");
  const program = new Program(idl as anchor.Idl, programId, provider);

  // Create mint and claimant
  const mint = await Token.createMint(
    connection,
    admin,
    admin.publicKey,
    null,
    0,
    TOKEN_PROGRAM_ID
  );
  const claimant = web3.Keypair.generate();
  await connection.requestAirdrop(claimant.publicKey, web3.LAMPORTS_PER_SOL);
  const claimantAta = await getAssociatedTokenAddress(mint.publicKey, claimant.publicKey);

  // Derive PDAs
  const version = new BN(0);
  const [distributorPda, bump] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("MerkleDistributor"), mint.publicKey.toBuffer(), version.toArrayLike(Buffer, "le", 8)],
    program.programId
  );
  const tokenVault = await getAssociatedTokenAddress(mint.publicKey, distributorPda, true);
  const [claimStatusPda] = web3.PublicKey.findProgramAddressSync(
    [Buffer.from("ClaimStatus"), claimant.publicKey.toBuffer(), distributorPda.toBuffer()],
    program.programId
  );

  // Dummy root/leaf/proof for single-leaf tree
  const unlocked = new BN(1);
  const locked = new BN(0);
  const leaf = hashLeaf(claimant.publicKey, unlocked, locked);
  const root = leaf;
  const proof: number[][] = [];

  // Initialize distributor
  await program.methods
    .newDistributor(version, Array.from(root), new BN(1), new BN(1), new BN(0), new BN(1), new BN(2))
    .accounts({
      distributor: distributorPda,
      tokenVault,
      admin: admin.publicKey,
      mint: mint.publicKey,
      payer: admin.publicKey,
      systemProgram: web3.SystemProgram.programId,
      tokenProgram: TOKEN_PROGRAM_ID,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      rent: web3.SYSVAR_RENT_PUBKEY,
    })
    .signers([admin])
    .rpc();

  // Mint tokens to vault
  await mint.mintTo(tokenVault, admin, [], 1);

  // Claim
  await program.methods
    .newClaim(Array.from(root), Array.from(leaf), Array.from(leaf), unlocked, locked, proof)
    .accounts({
      claimant: claimant.publicKey,
      claimantAta,
      distributor: distributorPda,
      tokenVault,
      claimStatus: claimStatusPda,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: web3.SystemProgram.programId,
    })
    .signers([claimant])
    .rpc();

  const account = await program.account.claimStatus.fetch(claimStatusPda);
  console.log("claim status", account);
})();
