from __future__ import annotations
import typing
from solders.pubkey import Pubkey
from solders.system_program import ID as SYS_PROGRAM_ID
from spl.token.constants import TOKEN_PROGRAM_ID, ASSOCIATED_TOKEN_PROGRAM_ID
from solders.instruction import Instruction, AccountMeta
import borsh_construct as borsh
from ..program_id import PROGRAM_ID


class NewDistributorArgs(typing.TypedDict):
    version: int
    root: list[int]
    max_total_claim: int
    max_num_nodes: int
    start_vesting_ts: int
    end_vesting_ts: int
    clawback_start_ts: int


layout = borsh.CStruct(
    "version" / borsh.U8,
    "root" / borsh.U8[32],
    "max_total_claim" / borsh.U64,
    "max_num_nodes" / borsh.U64,
    "start_vesting_ts" / borsh.I64,
    "end_vesting_ts" / borsh.I64,
    "clawback_start_ts" / borsh.I64,
)


class NewDistributorAccounts(typing.TypedDict):
    distributor: Pubkey
    clawback_receiver: Pubkey
    mint: Pubkey
    token_vault: Pubkey
    admin: Pubkey


def new_distributor(
    args: NewDistributorArgs,
    accounts: NewDistributorAccounts,
    program_id: Pubkey = PROGRAM_ID,
    remaining_accounts: typing.Optional[typing.List[AccountMeta]] = None,
) -> Instruction:
    keys: list[AccountMeta] = [
        AccountMeta(pubkey=accounts["distributor"], is_signer=False, is_writable=True),
        AccountMeta(
            pubkey=accounts["clawback_receiver"], is_signer=False, is_writable=True
        ),
        AccountMeta(pubkey=accounts["mint"], is_signer=False, is_writable=False),
        AccountMeta(pubkey=accounts["token_vault"], is_signer=False, is_writable=True),
        AccountMeta(pubkey=accounts["admin"], is_signer=True, is_writable=True),
        AccountMeta(pubkey=SYS_PROGRAM_ID, is_signer=False, is_writable=False),
        AccountMeta(
            pubkey=ASSOCIATED_TOKEN_PROGRAM_ID, is_signer=False, is_writable=False
        ),
        AccountMeta(pubkey=TOKEN_PROGRAM_ID, is_signer=False, is_writable=False),
    ]
    if remaining_accounts is not None:
        keys += remaining_accounts
    identifier = b" \x8bp\xab\x00\x02\xe1\x9b"
    encoded_args = layout.build(
        {
            "version": args["version"],
            "root": args["root"],
            "max_total_claim": args["max_total_claim"],
            "max_num_nodes": args["max_num_nodes"],
            "start_vesting_ts": args["start_vesting_ts"],
            "end_vesting_ts": args["end_vesting_ts"],
            "clawback_start_ts": args["clawback_start_ts"],
        }
    )
    data = identifier + encoded_args
    return Instruction(program_id, data, keys)
