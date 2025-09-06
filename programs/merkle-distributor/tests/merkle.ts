import * as anchor from "@coral-xyz/anchor";
import { Program, BN, web3 } from "@coral-xyz/anchor";
import { MerkleDistributor } from "../../target/types/merkle_distributor";
import {
  createMint,
  getAssociatedTokenAddress,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
  ASSOCIATED_TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import { keccak_256 } from "js-sha3";
import assert from "assert";

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

describe("merkle distributor", () => {
  const provider = anchor.AnchorProvider.local();
  anchor.setProvider(provider);
  const program = anchor.workspace
    .MerkleDistributor as Program<MerkleDistributor>;
  const connection = provider.connection;
  const admin = provider.wallet.payer as web3.Keypair;

  it("claims with valid proof", async () => {
    const version = new BN(0);
    const mint = await createMint(
      connection,
      admin,
      admin.publicKey,
      null,
      0
    );

    const claimant = web3.Keypair.generate();
    await connection.requestAirdrop(claimant.publicKey, web3.LAMPORTS_PER_SOL);
    const claimantAta = await getOrCreateAssociatedTokenAccount(
      connection,
      admin,
      mint,
      claimant.publicKey
    );

    const clawbackAta = await getOrCreateAssociatedTokenAccount(
      connection,
      admin,
      mint,
      admin.publicKey
    );

    const [distributorPda, bump] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("MerkleDistributor"),
        mint.toBuffer(),
        version.toArrayLike(Buffer, "le", 8),
      ],
      program.programId
    );
    const tokenVault = await getAssociatedTokenAddress(
      mint,
      distributorPda,
      true
    );

    const now = Math.floor(Date.now() / 1000);
    const startTs = new BN(now + 5);
    const endTs = new BN(now + 10);
    const clawbackTs = new BN(now + 10 + 86400);

    const amountUnlocked = new BN(5);
    const amountLocked = new BN(0);
    const leafClaimant = hashLeaf(
      claimant.publicKey,
      amountUnlocked,
      amountLocked
    );
    const other = web3.Keypair.generate();
    const leafOther = hashLeaf(other.publicKey, new BN(0), new BN(0));
    const root = hashNode(leafClaimant, leafOther);
    const proof = [Array.from(leafOther)];

    await program.methods
      .newDistributor(
        version,
        Array.from(root),
        amountUnlocked,
        new BN(2),
        startTs,
        endTs,
        clawbackTs
      )
      .accounts({
        distributor: distributorPda,
        clawbackReceiver: clawbackAta.address,
        mint,
        tokenVault,
        admin: admin.publicKey,
        systemProgram: web3.SystemProgram.programId,
        associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    await mintTo(
      connection,
      admin,
      mint,
      tokenVault,
      admin,
      amountUnlocked.toNumber()
    );

    const [claimStatus] = web3.PublicKey.findProgramAddressSync(
      [
        Buffer.from("ClaimStatus"),
        claimant.publicKey.toBuffer(),
        distributorPda.toBuffer(),
      ],
      program.programId
    );

    await program.methods
      .newClaim(amountUnlocked, amountLocked, proof)
      .accounts({
        distributor: distributorPda,
        claimStatus,
        from: tokenVault,
        to: claimantAta.address,
        claimant: claimant.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: web3.SystemProgram.programId,
      })
      .signers([claimant])
      .rpc();

    const dist = await program.account.merkleDistributor.fetch(
      distributorPda
    );
    assert.equal(dist.totalAmountClaimed.toNumber(), amountUnlocked.toNumber());
    assert.equal(dist.numNodesClaimed.toNumber(), 1);

    const status = await program.account.claimStatus.fetch(claimStatus);
    assert.equal(status.unlockedAmount.toNumber(), amountUnlocked.toNumber());

    const balance = await connection.getTokenAccountBalance(
      claimantAta.address
    );
    assert.equal(parseInt(balance.value.amount), amountUnlocked.toNumber());
  });
});
