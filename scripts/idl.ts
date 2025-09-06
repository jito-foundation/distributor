import { AnchorProvider, Program } from "@coral-xyz/anchor";
import { PublicKey } from "@solana/web3.js";
import MerkleDistributorIDL from "../target/idl/merkle_distributor.json" assert { type: "json" };
import { MERKLE_DISTRIBUTOR_PROGRAM_ID } from "./env";

export type MerkleDistributorIDLType = typeof MerkleDistributorIDL;

// Returns an Anchor Program for the Merkle distributor.
export function getMerkleDistributorProgram(
  provider: AnchorProvider = AnchorProvider.env(),
): Program<MerkleDistributorIDLType> {
  return new Program<MerkleDistributorIDLType>(
    MerkleDistributorIDL as MerkleDistributorIDLType,
    MERKLE_DISTRIBUTOR_PROGRAM_ID,
    provider,
  );
}
