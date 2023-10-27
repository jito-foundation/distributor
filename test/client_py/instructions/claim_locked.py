from __future__ import annotations
import typing
from solders.pubkey import Pubkey
from spl.token.constants import TOKEN_PROGRAM_ID
from solders.instruction import Instruction, AccountMeta
from ..program_id import PROGRAM_ID


class ClaimLockedAccounts(typing.TypedDict):
    distributor: Pubkey
    claim_status: Pubkey
    from_: Pubkey
    to: Pubkey
    claimant: Pubkey


def claim_locked(
    accounts: ClaimLockedAccounts,
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
    ]
    if remaining_accounts is not None:
        keys += remaining_accounts
    identifier = b'"\xce\xb5\x17\x0b\xcf\x93Z'
    encoded_args = b""
    data = identifier + encoded_args
    return Instruction(program_id, data, keys)
