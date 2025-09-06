use anchor_lang::{account, context::Context, prelude::*, Accounts, Key, ToAccountInfo};
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

use crate::{error::ErrorCode, state::merkle_distributor::MerkleDistributor};

const SECONDS_PER_HOUR: i64 = 3600; // 60 minutes * 60 seconds
const HOURS_PER_DAY: i64 = 24;
const SECONDS_PER_DAY: i64 = SECONDS_PER_HOUR * HOURS_PER_DAY; // 24 hours * 3600 seconds

/// Accounts for [merkle_distributor::handle_new_distributor].
#[derive(Accounts)]
#[instruction(version: u64)]
pub struct NewDistributor<'info> {
    /// [MerkleDistributor].
    #[account(
        init,
        seeds = [
            b"MerkleDistributor".as_ref(),
            mint.key().to_bytes().as_ref(),
            version.to_le_bytes().as_ref()
        ],
        bump,
        space = MerkleDistributor::LEN,
        payer = admin
    )]
    pub distributor: Account<'info, MerkleDistributor>,

    /// Clawback receiver token account
    #[account(mut, token::mint = mint)]
    pub clawback_receiver: Account<'info, TokenAccount>,

    /// The mint to distribute.
    pub mint: Account<'info, Mint>,

    /// Token vault
    #[account(
        init,
        associated_token::mint = mint,
        associated_token::authority=distributor,
        payer = admin,
    )]
    pub token_vault: Account<'info, TokenAccount>,

    /// Admin wallet, responsible for creating the distributor and paying for the transaction.
    /// Also has the authority to set the clawback receiver and change itself.
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The [System] program.
    pub system_program: Program<'info, System>,

    /// The [Associated Token] program.
    pub associated_token_program: Program<'info, AssociatedToken>,

    /// The [Token] program.
    pub token_program: Program<'info, Token>,
}

/// Creates a new [MerkleDistributor].
/// After creating this [MerkleDistributor],
/// the token_vault should be seeded with max_total_claim tokens.
/// CHECK:
///     1. The start timestamp is before the end timestamp
///     2. The clawback timestamp is after the end timestamp
///     3. The start, end, and clawback_start timestamps are all in the future
///     4. The clawback start is at least one day after end timestamp
#[allow(clippy::too_many_arguments)]
#[allow(clippy::result_large_err)]
pub fn handle_new_distributor(
    ctx: Context<NewDistributor>,
    version: u64,
    root: [u8; 32],
    max_total_claim: u64,
    max_num_nodes: u64,
    start_vesting_ts: i64,
    end_vesting_ts: i64,
    clawback_start_ts: i64,
) -> Result<()> {
    let curr_ts = Clock::get()?.unix_timestamp;

    require!(
        start_vesting_ts < end_vesting_ts,
        ErrorCode::StartTimestampAfterEnd
    );
    // New distributor parameters must all be set in the future
    require!(
        start_vesting_ts > curr_ts && end_vesting_ts > curr_ts && clawback_start_ts > curr_ts,
        ErrorCode::TimestampsNotInFuture
    );

    require!(
        clawback_start_ts > end_vesting_ts,
        ErrorCode::ClawbackDuringVesting
    );

    // Ensure clawback_start_ts is at least one day after end_vesting_ts
    require!(
        clawback_start_ts
            >= end_vesting_ts
                .checked_add(SECONDS_PER_DAY)
                .ok_or(ErrorCode::ArithmeticError)?,
        ErrorCode::InsufficientClawbackDelay
    );

    let distributor = &mut ctx.accounts.distributor;

    distributor.bump = ctx.bumps.distributor;
    distributor.version = version;
    distributor.root = root;
    distributor.mint = ctx.accounts.mint.key();
    distributor.token_vault = ctx.accounts.token_vault.key();
    distributor.max_total_claim = max_total_claim;
    distributor.max_num_nodes = max_num_nodes;
    distributor.total_amount_claimed = 0;
    distributor.num_nodes_claimed = 0;
    distributor.start_ts = start_vesting_ts;
    distributor.end_ts = end_vesting_ts;
    distributor.clawback_start_ts = clawback_start_ts;
    distributor.clawback_receiver = ctx.accounts.clawback_receiver.key();
    distributor.admin = ctx.accounts.admin.key();
    distributor.clawed_back = false;

    // Note: might get truncated, do not rely on
    msg! {
        "New distributor created with version = {}, mint={}, vault={} max_total_claim={}, max_nodes: {}, start_ts: {}, end_ts: {}, clawback_start: {}, clawback_receiver: {}",
            distributor.version,
            distributor.mint,
            ctx.accounts.token_vault.key(),
            distributor.max_total_claim,
            distributor.max_num_nodes,
            distributor.start_ts,
            distributor.end_ts,
            distributor.clawback_start_ts,
            distributor.clawback_receiver
    };

    Ok(())
}
