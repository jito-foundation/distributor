from __future__ import annotations
import typing
from solders.pubkey import Pubkey
from solders.instruction import Instruction, AccountMeta
from ..program_id import PROGRAM_ID


class SetAminAccounts(typing.TypedDict):
    distributor: Pubkey
    admin: Pubkey
    new_admin: Pubkey


def set_amin(
    accounts: SetAminAccounts,
    program_id: Pubkey = PROGRAM_ID,
    remaining_accounts: typing.Optional[typing.List[AccountMeta]] = None,
) -> Instruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["distributor"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["admin"], is_signer=True, is_writable=True),
        AccountMeta(pubkey=accounts["new_admin"], is_signer=False, is_writable=True),
    ]
    if remaining_accounts is not None:
        keys += remaining_accounts
    identifier = b"\xb0\x054 \x16\x18\xa7\xf9"
    encoded_args = b""
    data = identifier + encoded_args
    return Instruction(program_id, data, keys)
