from __future__ import annotations
import typing
from solders.pubkey import Pubkey
from solders.system_program import ID as SYS_PROGRAM_ID
from spl.token.constants import TOKEN_PROGRAM_ID
from solders.instruction import Instruction, AccountMeta
from construct import Construct
import borsh_construct as borsh
from ..program_id import PROGRAM_ID


class NewClaimArgs(typing.TypedDict):
    amount_unlocked: int
    amount_locked: int
    proof: list[list[int]]


layout = borsh.CStruct(
    "amount_unlocked" / borsh.U64,
    "amount_locked" / borsh.U64,
    "proof" / borsh.Vec(typing.cast(Construct, borsh.U8[32])),
)


class NewClaimAccounts(typing.TypedDict):
    distributor: Pubkey
    claim_status: Pubkey
    from_: Pubkey
    to: Pubkey
    claimant: Pubkey


def new_claim(
    args: NewClaimArgs,
    accounts: NewClaimAccounts,
    program_id: Pubkey = PROGRAM_ID,
    remaining_accounts: typing.Optional[typing.List[AccountMeta]] = None,
) -> Instruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["distributor"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["claim_status"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["from_"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["to"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["claimant"], is_signer=True, is_writable=True),
        AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
        AccountMeta(pubkey=SYS_PROGRAM_ID, is_signer=False, is_writable=False),
    ]
    if remaining_accounts is not None:
        keys += remaining_accounts
    identifier = b"N\xb1b{\xd2\x15\xbbS"
    encoded_args = layout.build(
        {
            "amount_unlocked": args["amount_unlocked"],
            "amount_locked": args["amount_locked"],
            "proof": args["proof"],
        }
    )
    data = identifier + encoded_args
    return Instruction(program_id, data, keys)
