import { keccak_256 } from "js-sha3";
import { PublicKey } from "@solana/web3.js";

// Builds a leaf hash using the on-chain encoding:
// keccak(0x00 || claimant || amount_unlocked_le || amount_locked_le)
export function leafFor(
  claimant: PublicKey,
  amountUnlocked: bigint,
  amountLocked: bigint,
): Buffer {
  const data = Buffer.alloc(1 + 32 + 8 + 8);
  data[0] = 0x00;
  claimant.toBuffer().copy(data, 1);
  data.writeBigUInt64LE(amountUnlocked, 33);
  data.writeBigUInt64LE(amountLocked, 41);
  return Buffer.from(keccak_256.arrayBuffer(data));
}

function hashNode(left: Buffer, right: Buffer): Buffer {
  const node = Buffer.concat([Buffer.from([0x01]), left, right]);
  return Buffer.from(keccak_256.arrayBuffer(node));
}

// Builds a simple binary Merkle tree where the last node is duplicated
// if a level has an odd number of elements.
export function buildTree(leaves: Buffer[]) {
  const levels: Buffer[][] = [leaves.map((l) => Buffer.from(l))];
  while (levels[levels.length - 1].length > 1) {
    const prev = levels[levels.length - 1];
    const next: Buffer[] = [];
    for (let i = 0; i < prev.length; i += 2) {
      const left = prev[i];
      const right = i + 1 < prev.length ? prev[i + 1] : prev[i];
      next.push(hashNode(left, right));
    }
    levels.push(next);
  }
  return {
    root: levels[levels.length - 1][0],
    getProof(idx: number): Buffer[] {
      const proof: Buffer[] = [];
      for (let level = 0; level < levels.length - 1; level++) {
        const nodes = levels[level];
        const isRight = idx % 2 === 1;
        const pairIdx = isRight ? idx - 1 : idx + 1;
        if (pairIdx < nodes.length) {
          proof.push(nodes[pairIdx]);
        }
        idx = Math.floor(idx / 2);
      }
      return proof;
    },
  };
}
