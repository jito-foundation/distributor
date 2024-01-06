use anchor_lang::{context::Context, prelude::*, Accounts, Key, Result};

use crate::state::merkle_distributor::MerkleDistributor;

/// Accounts for [merkle_distributor::enable_pool].
#[derive(Accounts)]
pub struct UpdatePoolStatus<'info> {
    /// [MerkleDistributor].
    #[account(
        mut,
        has_one = admin,
    )]
    pub distributor: Account<'info, MerkleDistributor>,

    /// Payer to create the distributor.
    #[account(mut)]
    pub admin: Signer<'info>,
}

/// Enable pool for claim
#[allow(clippy::result_large_err)]
pub fn handle_enable(ctx: Context<UpdatePoolStatus>) -> Result<()> {
    let distributor = &mut ctx.accounts.distributor;
    distributor.is_enable = true;
    Ok(())
}

/// Disable pool for claim
#[allow(clippy::result_large_err)]
pub fn handle_disable(ctx: Context<UpdatePoolStatus>) -> Result<()> {
    let distributor = &mut ctx.accounts.distributor;
    distributor.is_enable = false;
    Ok(())
}
