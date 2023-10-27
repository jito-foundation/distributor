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


class ClaimStatusJSON(typing.TypedDict):
    claimant: str
    locked_amount: int
    locked_amount_withdrawn: int
    unlocked_amount: int


@dataclass
class ClaimStatus:
    discriminator: typing.ClassVar = b"\x16\xb7\xf9\x9d\xf7_\x96`"
    layout: typing.ClassVar = borsh.CStruct(
        "claimant" / BorshPubkey,
        "locked_amount" / borsh.U64,
        "locked_amount_withdrawn" / borsh.U64,
        "unlocked_amount" / borsh.U64,
    )
    claimant: Pubkey
    locked_amount: int
    locked_amount_withdrawn: int
    unlocked_amount: int

    @classmethod
    async def fetch(
        cls,
        conn: AsyncClient,
        address: Pubkey,
        commitment: typing.Optional[Commitment] = None,
        program_id: Pubkey = PROGRAM_ID,
    ) -> typing.Optional["ClaimStatus"]:
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
    ) -> typing.List[typing.Optional["ClaimStatus"]]:
        infos = await get_multiple_accounts(conn, addresses, commitment=commitment)
        res: typing.List[typing.Optional["ClaimStatus"]] = []
        for info in infos:
            if info is None:
                res.append(None)
                continue
            if info.account.owner != program_id:
                raise ValueError("Account does not belong to this program")
            res.append(cls.decode(info.account.data))
        return res

    @classmethod
    def decode(cls, data: bytes) -> "ClaimStatus":
        if data[:ACCOUNT_DISCRIMINATOR_SIZE] != cls.discriminator:
            raise AccountInvalidDiscriminator(
                "The discriminator for this account is invalid"
            )
        dec = ClaimStatus.layout.parse(data[ACCOUNT_DISCRIMINATOR_SIZE:])
        return cls(
            claimant=dec.claimant,
            locked_amount=dec.locked_amount,
            locked_amount_withdrawn=dec.locked_amount_withdrawn,
            unlocked_amount=dec.unlocked_amount,
        )

    def to_json(self) -> ClaimStatusJSON:
        return {
            "claimant": str(self.claimant),
            "locked_amount": self.locked_amount,
            "locked_amount_withdrawn": self.locked_amount_withdrawn,
            "unlocked_amount": self.unlocked_amount,
        }

    @classmethod
    def from_json(cls, obj: ClaimStatusJSON) -> "ClaimStatus":
        return cls(
            claimant=Pubkey.from_string(obj["claimant"]),
            locked_amount=obj["locked_amount"],
            locked_amount_withdrawn=obj["locked_amount_withdrawn"],
            unlocked_amount=obj["unlocked_amount"],
        )
