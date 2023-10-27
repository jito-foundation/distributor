import typing
from dataclasses import dataclass
from solders.pubkey import Pubkey
from solana.rpc.async_api import AsyncClient
from solana.rpc.commitment import Commitment
import borsh_construct as borsh
from anchorpy.coder.accounts import ACCOUNT_DISCRIMINATOR_SIZE
from anchorpy.error import AccountInvalidDiscriminator
from anchorpy.utils.rpc import get_multiple_accounts
from anchorpy.borsh_extension import BorshPubkey
from ..program_id import PROGRAM_ID


class MerkleDistributorJSON(typing.TypedDict):
    bump: int
    version: int
    root: list[int]
    mint: str
    token_vault: str
    max_total_claim: int
    max_num_nodes: int
    total_amount_claimed: int
    num_nodes_claimed: int
    start_ts: int
    end_ts: int
    clawback_start_ts: int
    clawback_receiver: str
    admin: str
    clawed_back: bool


@dataclass
class MerkleDistributor:
    discriminator: typing.ClassVar = b"Mw\x8bFT\xf7\x0c\x1a"
    layout: typing.ClassVar = borsh.CStruct(
        "bump" / borsh.U8,
        "version" / borsh.U8,
        "root" / borsh.U8[32],
        "mint" / BorshPubkey,
        "token_vault" / BorshPubkey,
        "max_total_claim" / borsh.U64,
        "max_num_nodes" / borsh.U64,
        "total_amount_claimed" / borsh.U64,
        "num_nodes_claimed" / borsh.U64,
        "start_ts" / borsh.I64,
        "end_ts" / borsh.I64,
        "clawback_start_ts" / borsh.I64,
        "clawback_receiver" / BorshPubkey,
        "admin" / BorshPubkey,
        "clawed_back" / borsh.Bool,
    )
    bump: int
    version: int
    root: list[int]
    mint: Pubkey
    token_vault: Pubkey
    max_total_claim: int
    max_num_nodes: int
    total_amount_claimed: int
    num_nodes_claimed: int
    start_ts: int
    end_ts: int
    clawback_start_ts: int
    clawback_receiver: Pubkey
    admin: Pubkey
    clawed_back: bool

    @classmethod
    async def fetch(
        cls,
        conn: AsyncClient,
        address: Pubkey,
        commitment: typing.Optional[Commitment] = None,
        program_id: Pubkey = PROGRAM_ID,
    ) -> typing.Optional["MerkleDistributor"]:
        resp = await conn.get_account_info(address, commitment=commitment)
        info = resp.value
        if info is None:
            return None
        if info.owner != program_id:
            raise ValueError("Account does not belong to this program")
        bytes_data = info.data
        return cls.decode(bytes_data)

    @classmethod
    async def fetch_multiple(
        cls,
        conn: AsyncClient,
        addresses: list[Pubkey],
        commitment: typing.Optional[Commitment] = None,
        program_id: Pubkey = PROGRAM_ID,
    ) -> typing.List[typing.Optional["MerkleDistributor"]]:
        infos = await get_multiple_accounts(conn, addresses, commitment=commitment)
        res: typing.List[typing.Optional["MerkleDistributor"]] = []
        for info in infos:
            if info is None:
                res.append(None)
                continue
            if info.account.owner != program_id:
                raise ValueError("Account does not belong to this program")
            res.append(cls.decode(info.account.data))
        return res

    @classmethod
    def decode(cls, data: bytes) -> "MerkleDistributor":
        if data[:ACCOUNT_DISCRIMINATOR_SIZE] != cls.discriminator:
            raise AccountInvalidDiscriminator(
                "The discriminator for this account is invalid"
            )
        dec = MerkleDistributor.layout.parse(data[ACCOUNT_DISCRIMINATOR_SIZE:])
        return cls(
            bump=dec.bump,
            version=dec.version,
            root=dec.root,
            mint=dec.mint,
            token_vault=dec.token_vault,
            max_total_claim=dec.max_total_claim,
            max_num_nodes=dec.max_num_nodes,
            total_amount_claimed=dec.total_amount_claimed,
            num_nodes_claimed=dec.num_nodes_claimed,
            start_ts=dec.start_ts,
            end_ts=dec.end_ts,
            clawback_start_ts=dec.clawback_start_ts,
            clawback_receiver=dec.clawback_receiver,
            admin=dec.admin,
            clawed_back=dec.clawed_back,
        )

    def to_json(self) -> MerkleDistributorJSON:
        return {
            "bump": self.bump,
            "version": self.version,
            "root": self.root,
            "mint": str(self.mint),
            "token_vault": str(self.token_vault),
            "max_total_claim": self.max_total_claim,
            "max_num_nodes": self.max_num_nodes,
            "total_amount_claimed": self.total_amount_claimed,
            "num_nodes_claimed": self.num_nodes_claimed,
            "start_ts": self.start_ts,
            "end_ts": self.end_ts,
            "clawback_start_ts": self.clawback_start_ts,
            "clawback_receiver": str(self.clawback_receiver),
            "admin": str(self.admin),
            "clawed_back": self.clawed_back,
        }

    @classmethod
    def from_json(cls, obj: MerkleDistributorJSON) -> "MerkleDistributor":
        return cls(
            bump=obj["bump"],
            version=obj["version"],
            root=obj["root"],
            mint=Pubkey.from_string(obj["mint"]),
            token_vault=Pubkey.from_string(obj["token_vault"]),
            max_total_claim=obj["max_total_claim"],
            max_num_nodes=obj["max_num_nodes"],
            total_amount_claimed=obj["total_amount_claimed"],
            num_nodes_claimed=obj["num_nodes_claimed"],
            start_ts=obj["start_ts"],
            end_ts=obj["end_ts"],
            clawback_start_ts=obj["clawback_start_ts"],
            clawback_receiver=Pubkey.from_string(obj["clawback_receiver"]),
            admin=Pubkey.from_string(obj["admin"]),
            clawed_back=obj["clawed_back"],
        )
