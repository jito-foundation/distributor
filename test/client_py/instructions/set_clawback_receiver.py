from __future__ import annotations
import typing
from solders.pubkey import Pubkey
from solders.instruction import Instruction, AccountMeta
from ..program_id import PROGRAM_ID


class SetClawbackReceiverAccounts(typing.TypedDict):
    distributor: Pubkey
    new_clawback_account: Pubkey
    admin: Pubkey


def set_clawback_receiver(
    accounts: SetClawbackReceiverAccounts,
    program_id: Pubkey = PROGRAM_ID,
    remaining_accounts: typing.Optional[typing.List[AccountMeta]] = None,
) -> Instruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["distributor"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["new_clawback_account"], is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=accounts["admin"], is_signer=True, is_writable=True),
    ]
    if remaining_accounts is not None:
        keys += remaining_accounts
    identifier = b'\x99\xd9"\x14\x13\x1d\xe5K'
    encoded_args = b""
    data = identifier + encoded_args
    return Instruction(program_id, data, keys)
