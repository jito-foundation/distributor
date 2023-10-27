use anchor_lang::{
    accounts::{account::Account, signer::Signer},
    context::Context,
    prelude::*,
    Accounts, Result,
};

use crate::{error::ErrorCode, state::merkle_distributor::MerkleDistributor};

/// [merkle_distributor::set_clawback_receiver] accounts.
#[derive(Accounts)]
pub struct SetAdmin<'info> {
    /// The [MerkleDistributor].
    #[account(mut)]
    pub distributor: Account<'info, MerkleDistributor>,

    /// Admin signer
    #[account(mut, address = distributor.admin @ ErrorCode::Unauthorized)]
    pub admin: Signer<'info>,

    /// New admin account
    /// CHECK: this can be any new account
    #[account(mut)]
    pub new_admin: AccountInfo<'info>,
}

/// Sets new admin account
/// CHECK:
///     1. The new admin is not the same as the old one
#[allow(clippy::result_large_err)]
pub fn handle_set_admin(ctx: Context<SetAdmin>) -> Result<()> {
    let distributor = &mut ctx.accounts.distributor;

    require!(
        ctx.accounts.admin.key != &ctx.accounts.new_admin.key(),
        ErrorCode::SameAdmin
    );

    distributor.admin = ctx.accounts.new_admin.key();

    // Note: might get truncated, do not rely on
    msg!("set new admin to {}", ctx.accounts.new_admin.key());

    Ok(())
}
