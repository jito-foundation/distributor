use anchor_lang::{context::Context, prelude::*, Accounts, Key, Result};

use crate::state::merkle_distributor::MerkleDistributor;

/// Accounts for [merkle_distributor::set_enable_slot].
#[derive(Accounts)]
pub struct SetEnableSlot<'info> {
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

/// set enable slot
#[allow(clippy::result_large_err)]
pub fn handle_set_enable_slot(ctx: Context<SetEnableSlot>, enable_slot: u64) -> Result<()> {
    let distributor = &mut ctx.accounts.distributor;
    distributor.enable_slot = enable_slot;
    Ok(())
}
