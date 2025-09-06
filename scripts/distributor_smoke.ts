import { AnchorProvider, BN } from "@coral-xyz/anchor";
import {
  ASSOCIATED_TOKEN_PROGRAM_ID,
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  transfer as tokenTransfer,
} from "@solana/spl-token";
import { Keypair, SystemProgram } from "@solana/web3.js";
import { getProvider } from "./connection";
import { getMerkleDistributorProgram } from "./idl";
import { MERKLE_DISTRIBUTOR_PROGRAM_ID, TOKEN_PROGRAM_ID } from "./env";
import {
  distributorPda,
  tokenVaultPda,
  claimStatusPda,
} from "./pdas";
import { leafFor, buildTree } from "./merkle";
import { assert, assertEqual } from "./asserts";

function sleep(ms: number) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

(async () => {
  const provider = getProvider();
  AnchorProvider.env();
  const program = getMerkleDistributorProgram(provider);
  const connection = provider.connection;
  const payer = provider.wallet.payer as Keypair;

  console.log("Payer", payer.publicKey.toBase58());

  // 1. Create mint and fund payer
  const mint = await createMint(
    connection,
    payer,
    payer.publicKey,
    null,
    9,
    undefined,
    {},
    TOKEN_PROGRAM_ID,
  );
  const payerAta = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mint,
    payer.publicKey,
  );
  await mintTo(connection, payer, mint, payerAta.address, payer, 1_000_000n);

  // 2. Prepare claimants and Merkle tree
  const extra1 = Keypair.generate();
  const extra2 = Keypair.generate();
  const claimants = [
    { kp: payer, unlocked: 5n, locked: 2n },
    { kp: extra1, unlocked: 3n, locked: 1n },
    { kp: extra2, unlocked: 1n, locked: 0n },
  ];
  const leaves = claimants.map((c) =>
    leafFor(c.kp.publicKey, BigInt(c.unlocked), BigInt(c.locked)),
  );
  const tree = buildTree(leaves);
  console.log("Merkle root", tree.root.toString("hex"));

  // 3. Derive PDAs
  const version = 1n;
  const distributor = distributorPda(mint, version);
  const tokenVault = tokenVaultPda(distributor, mint);
  const claimantAta = payerAta.address; // payer claims to own ATA

  // 4. Initialize distributor
  const now = Math.floor(Date.now() / 1000);
  const startTs = new BN(now + 60);
  const endTs = new BN(now + 60 + 86400);
  const clawbackTs = new BN(endTs.toNumber() + 86400);

  await program.methods
    .newDistributor(
      new BN(version),
      Array.from(tree.root),
      new BN(1_000_000),
      new BN(claimants.length),
      startTs,
      endTs,
      clawbackTs,
    )
    .accounts({
      distributor,
      tokenVault,
      admin: payer.publicKey,
      mint,
      systemProgram: SystemProgram.programId,
      associatedTokenProgram: ASSOCIATED_TOKEN_PROGRAM_ID,
      tokenProgram: TOKEN_PROGRAM_ID,
    })
    .signers([payer])
    .rpc();
  console.log("Distributor initialized", distributor.toBase58());

  // 5. Fund vault by transferring from payer ATA
  await tokenTransfer(
    connection,
    payer,
    payerAta.address,
    tokenVault,
    payer.publicKey,
    1_000_000n,
  );

  // 6. Claim for payer
  const claimantIndex = 0;
  const proof = tree
    .getProof(claimantIndex)
    .map((p) => Array.from(p));
  const amountUnlocked = new BN(claimants[claimantIndex].unlocked);
  const amountLocked = new BN(claimants[claimantIndex].locked);
  const claimStatus = claimStatusPda(payer.publicKey, distributor);

  const tx = await program.methods
    .newClaim(amountUnlocked, amountLocked, proof)
    .accounts({
      distributor,
      claimStatus,
      from: tokenVault,
      to: claimantAta,
      claimant: payer.publicKey,
      tokenProgram: TOKEN_PROGRAM_ID,
      systemProgram: SystemProgram.programId,
    })
    .signers([payer])
    .rpc();
  console.log("Claim tx", tx);
  const txInfo = await connection.getTransaction(tx, {
    commitment: "confirmed",
  });
  const events = program.coder.events.parseLogs(txInfo?.meta?.logMessages || []);
  events.forEach((e) => console.log("Event", e.name, e.data));

  // 7. Attempt to claim locked tokens immediately (likely zero)
  try {
    const tx2 = await program.methods
      .claimLocked()
      .accounts({
        distributor,
        claimStatus,
        from: tokenVault,
        to: claimantAta,
        claimant: payer.publicKey,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([payer])
      .rpc();
    console.log("claim_locked tx", tx2);
  } catch (e: any) {
    console.log("claim_locked failed (expected if nothing vested)", e.error?.errorCode?.code || e);
  }

  // 8. Admin flows
  const newClawback = await getOrCreateAssociatedTokenAccount(
    connection,
    payer,
    mint,
    Keypair.generate().publicKey,
  );
  await program.methods
    .setClawbackReceiver()
    .accounts({
      distributor,
      newClawbackAccount: newClawback.address,
      admin: payer.publicKey,
    })
    .signers([payer])
    .rpc();

  const newAdmin = Keypair.generate();
  await program.methods
    .setAdmin()
    .accounts({
      distributor,
      admin: payer.publicKey,
      newAdmin: newAdmin.publicKey,
    })
    .signers([payer])
    .rpc();
  console.log("Admin changed to", newAdmin.publicKey.toBase58());

  // 9. After vesting end, clawback remaining funds
  console.log("Waiting for vesting to end...");
  await sleep(2000);
  try {
    await program.methods
      .clawback()
      .accounts({
        distributor,
        from: tokenVault,
        to: newClawback.address,
        claimant: payer.publicKey, // anyone can trigger
        systemProgram: SystemProgram.programId,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([payer])
      .rpc();
    console.log("Clawback executed");
  } catch (e: any) {
    console.log("clawback failed", e.error?.errorCode?.code || e);
  }

  const balance = await connection.getTokenAccountBalance(claimantAta);
  console.log("Final claimant balance", balance.value.uiAmountString);
})();
