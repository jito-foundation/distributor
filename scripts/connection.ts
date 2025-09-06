import { AnchorProvider, Wallet } from "@coral-xyz/anchor";
import { Connection, Keypair } from "@solana/web3.js";
import fs from "fs";
import os from "os";

// Returns a connection to the local validator or the URL specified by ANCHOR_PROVIDER_URL.
export function getConnection(): Connection {
  const url = process.env.ANCHOR_PROVIDER_URL || "http://localhost:8899";
  return new Connection(url, "confirmed");
}

// Loads the payer keypair from ANCHOR_WALLET or the default Solana CLI location.
export function getKeypair(): Keypair {
  const walletPath =
    process.env.ANCHOR_WALLET ||
    `${os.homedir()}/.config/solana/id.json`;
  const secret = JSON.parse(fs.readFileSync(walletPath, "utf-8"));
  return Keypair.fromSecretKey(Uint8Array.from(secret));
}

// Constructs an Anchor provider using the loaded keypair.
export function getProvider(): AnchorProvider {
  const connection = getConnection();
  const wallet = new Wallet(getKeypair());
  return new AnchorProvider(connection, wallet, {
    commitment: "confirmed",
  });
}
