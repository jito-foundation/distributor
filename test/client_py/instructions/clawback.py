from __future__ import annotations
import typing
from solders.pubkey import Pubkey
from solders.system_program import ID as SYS_PROGRAM_ID
from spl.token.constants import TOKEN_PROGRAM_ID
from solders.instruction import Instruction, AccountMeta
from ..program_id import PROGRAM_ID


class ClawbackAccounts(typing.TypedDict):
    distributor: Pubkey
    from_: Pubkey
    to: Pubkey
    claimant: Pubkey


def clawback(
    accounts: ClawbackAccounts,
    program_id: Pubkey = PROGRAM_ID,
    remaining_accounts: typing.Optional[typing.List[AccountMeta]] = None,
) -> Instruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["distributor"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["from_"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["to"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["claimant"], is_signer=True, is_writable=False),
        AccountMeta(pubkey=SYS_PROGRAM_ID, is_signer=False, is_writable=False),
        AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
    ]
    if remaining_accounts is not None:
        keys += remaining_accounts
    identifier = b"o\\\x8eO!\xeaR\x1b"
    encoded_args = b""
    data = identifier + encoded_args
    return Instruction(program_id, data, keys)
