from typing import List

from solders.account import Account
from solders.bankrun import ProgramTestContext
from solders.instruction import Instruction
from solders.message import Message
from solders.token.associated import get_associated_token_address
from dataclasses import dataclass
from time import time

from client_py.instructions.new_distributor import new_distributor
from merkle_tree import MerkleTree
from test_utils import get_distributor_pda
from client_py.instructions.clawback import clawback
from solders.clock import Clock

from solders.transaction import TransactionError, VersionedTransaction
from client_py.program_id import PROGRAM_ID
from solders.token.state import TokenAccount, TokenAccountState, Mint

from pathlib import Path
from pytest import mark, raises
from solders.bankrun import start_anchor
from solders.pubkey import Pubkey
from solders.keypair import Keypair

TOKEN_PROGRAM_ID = Pubkey.from_string("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA")


@dataclass
class TestHook:
    mint: Pubkey
    clawback_keypair: Keypair
    distributor_ata: Pubkey
    program_id: Pubkey
    context: ProgramTestContext


async def init_test_accounts(program_id) -> TestHook:
    mint_address = Pubkey.new_unique()
    clawback_keypair = Keypair()

    clawback_token_acc = TokenAccount(
        mint=mint_address,
        owner=clawback_keypair.pubkey(),
        amount=1_000,
        delegate=None,
        state=TokenAccountState.Initialized,
        is_native=None,
        delegated_amount=0,
        close_authority=None,
    )

    (distributor, bump) = get_distributor_pda(mint_address, program_id, 0)
    distributor_ata = get_associated_token_address(distributor, mint_address)

    accounts = [
        (
            get_associated_token_address(clawback_keypair.pubkey(), mint_address),
            Account(
                lamports=1_000_000_000,
                data=bytes(clawback_token_acc),
                owner=TOKEN_PROGRAM_ID,
                executable=False,
            ),
        ),
        (
            mint_address,
            Account(
                data=bytes(
                    Mint(
                        decimals=9,
                        mint_authority=None,
                        supply=10000,
                        is_initialized=True,
                    )
                ),
                lamports=100_000,
                owner=TOKEN_PROGRAM_ID,
            ),
        ),
    ]

    context = await start_anchor(Path("../"), accounts=accounts)
    return TestHook(
        mint_address, clawback_keypair, distributor_ata, program_id, context
    )


@mark.asyncio
async def test_new_distributor():
    """Test that a new distributor can successfully be created"""
    testhook = await init_test_accounts(PROGRAM_ID)
    context = testhook.context
    program_id = testhook.program_id
    mint = testhook.mint
    clawback_address = get_associated_token_address(
        testhook.clawback_keypair.pubkey(), mint
    )

    (distributor, bump) = get_distributor_pda(mint, program_id)
    payer = context.payer

    distributor = new_distributor(
        {
            "version": 0,
            "root": [0] * 32,
            "max_total_claim": 100_00_00,
            "max_num_nodes": 1,
            "start_vesting_ts": 1,
            "end_vesting_ts": 10,
            "clawback_start_ts": 11,
        },
        {
            "distributor": distributor,
            "token_vault": get_associated_token_address(distributor, mint),
            "clawback_receiver": clawback_address,
            "mint": mint,
            "admin": payer.pubkey(),
        },
    )

    ixs = [distributor]
    blockhash = context.last_blockhash
    msg = Message.new_with_blockhash(ixs, payer.pubkey(), blockhash)
    tx = VersionedTransaction(msg, [payer])
    client = context.banks_client
    await client.process_transaction(tx)


@mark.asyncio
async def test_clawback():
    """Test that an account can be clawed back successfully"""
    test_hook = await init_test_accounts(PROGRAM_ID)
    context = test_hook.context
    program_id = test_hook.program_id
    mint = test_hook.mint
    clawback_address = get_associated_token_address(
        test_hook.clawback_keypair.pubkey(), mint
    )
    distributor_ata = test_hook.distributor_ata

    # setup distributor
    (distributor, bump) = get_distributor_pda(mint, program_id, 0)
    payer = context.payer

    new_distributor_ix = new_distributor(
        {
            "version": 0,
            "root": [0] * 32,
            "max_total_claim": 100_00_00,
            "max_num_nodes": 1,
            "start_vesting_ts": 1,
            "end_vesting_ts": 10,
            "clawback_start_ts": 11,
        },
        {
            "distributor": distributor,
            "token_vault": get_associated_token_address(distributor, mint),
            "clawback_receiver": clawback_address,
            "mint": mint,
            "admin": payer.pubkey(),
        },
    )

    ixs = [new_distributor_ix]
    blockhash = context.last_blockhash
    msg = Message.new_with_blockhash(ixs, payer.pubkey(), blockhash)

    payer = context.payer
    clawback_ix = clawback(
        {
            "distributor": distributor,
            "from_": distributor_ata,
            "to": clawback_address,
            "claimant": test_hook.clawback_keypair.pubkey(),
        }
    )

    ixs = [new_distributor_ix, clawback_ix]
    blockhash = context.last_blockhash
    msg = Message.new_with_blockhash(ixs, payer.pubkey(), blockhash)
    tx = VersionedTransaction(msg, [payer, test_hook.clawback_keypair])
    client = context.banks_client
    await client.process_transaction(tx)


async def setup_clawback_test_case() -> (TestHook, List[Instruction]):
    """Setup a test case for clawback by initializing a new distributor"""
    test_hook = await init_test_accounts(PROGRAM_ID)
    context = test_hook.context
    program_id = test_hook.program_id
    mint = test_hook.mint
    clawback_address = get_associated_token_address(
        test_hook.clawback_keypair.pubkey(), mint
    )

    # setup distributor
    (distributor, bump) = get_distributor_pda(mint, program_id)
    curr_ts = int(time())

    new_distributor_ix = new_distributor(
        {
            "version": 0,
            "root": [0] * 32,
            "max_total_claim": 100_00_00,
            "max_num_nodes": 1,
            "start_vesting_ts": curr_ts,
            "end_vesting_ts": curr_ts + 100_000,
            "clawback_start_ts": curr_ts + 100_000 + 1,
        },
        {
            "distributor": distributor,
            "token_vault": get_associated_token_address(distributor, mint),
            "clawback_receiver": clawback_address,
            "mint": mint,
            "admin": context.payer.pubkey(),
        },
    )

    return test_hook, new_distributor_ix


@mark.asyncio
async def test_clawback_before_claim_expiry():
    """Test that clawback instruction fails before the claim expiry window"""
    test_hook, new_distributor_ix = await setup_clawback_test_case()
    context = test_hook.context

    clawback_address = get_associated_token_address(
        test_hook.clawback_keypair.pubkey(), test_hook.mint
    )

    clawback_ix = clawback(
        {
            "distributor": get_distributor_pda(test_hook.mint, test_hook.program_id, 0)[
                0
            ],
            "from_": test_hook.distributor_ata,
            "to": clawback_address,
            "claimant": test_hook.clawback_keypair.pubkey(),
        }
    )

    ixs = [new_distributor_ix, clawback_ix]
    blockhash = context.last_blockhash
    msg = Message.new_with_blockhash(ixs, context.payer.pubkey(), blockhash)
    tx = VersionedTransaction(msg, [context.payer, test_hook.clawback_keypair])
    client = context.banks_client

    with raises(TransactionError) as e:
        await client.process_transaction(tx)
    assert "Error processing Instruction 1: custom program error: 0x1778" in str(
        e.value
    )


@mark.asyncio
async def test_clawback_after():
    """Test that calling the Clawback instruction works after the clawback window begins"""
    test_hook, new_distributor_ix = await setup_clawback_test_case()
    context = test_hook.context

    clawback_address = get_associated_token_address(
        test_hook.clawback_keypair.pubkey(), test_hook.mint
    )

    curr_ts = int(time())
    client = context.banks_client
    # now warp time to 6 months after curr_ts
    six_months_from_now = curr_ts + 6 * 31 * 24 * 60 * 60
    current_clock = await client.get_clock()

    clawback_ix = clawback(
        {
            "distributor": get_distributor_pda(test_hook.mint, test_hook.program_id, 0)[
                0
            ],
            "from_": test_hook.distributor_ata,
            "to": clawback_address,
            "claimant": test_hook.clawback_keypair.pubkey(),
        }
    )

    # warp to one slot and 6 months after claim expiry window
    new_clock = Clock(
        slot=current_clock.slot + 1,
        epoch_start_timestamp=current_clock.epoch_start_timestamp,
        epoch=current_clock.epoch,
        leader_schedule_epoch=current_clock.leader_schedule_epoch,
        unix_timestamp=six_months_from_now,
    )
    context.set_clock(new_clock)

    ixs = [new_distributor_ix, clawback_ix]
    blockhash = context.last_blockhash
    msg = Message.new_with_blockhash(ixs, context.payer.pubkey(), blockhash)
    tx = VersionedTransaction(msg, [context.payer, test_hook.clawback_keypair])
    client = context.banks_client

    await client.process_transaction(tx)

@mark.asyncio
def test_claim():
    """Test that claim works correctly"""



@mark.asyncio
def test_load_merkle_tree():
    """Test that loading the merkle tree works correctly"""
    path = "merkle_tree.json"
    with open(path, "r") as f:
        json_str = f.read()

    merkle_tree = MerkleTree.from_json(json_str)
    assert merkle_tree.max_total_claim == 600000000000

