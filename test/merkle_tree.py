"""Helper methods to create a Merkle Tree"""
import json
from typing import List


class TreeNode:
    def __init__(
        self,
        claimant: List[int],
        amount_unlocked: int,
        amount_locked: int,
        proof: List[List[int]],
    ):
        self.claimant = claimant
        self.amount_unlocked = amount_unlocked
        self.amount_locked = amount_locked
        self.proof = proof


class MerkleTree:
    def __init__(
        self,
        merkle_root: List[int],
        max_num_nodes: int,
        max_total_claim: int,
        tree_nodes: List[TreeNode],
    ):
        self.merkle_root = merkle_root
        self.max_num_nodes = max_num_nodes
        self.max_total_claim = max_total_claim
        self.tree_nodes = tree_nodes

    @classmethod
    def from_json(cls, json_str: str) -> "MerkleTree":
        data = json.loads(json_str)
        tree_nodes = [TreeNode(**node) for node in data["tree_nodes"]]
        return cls(
            data["merkle_root"],
            data["max_num_nodes"],
            data["max_total_claim"],
            tree_nodes,
        )
