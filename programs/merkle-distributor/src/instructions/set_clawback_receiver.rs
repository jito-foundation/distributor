use anchor_lang::{
    accounts::{account::Account, signer::Signer},
    context::Context,
    prelude::*,
    Accounts, Result, ToAccountInfo,
};
use anchor_spl::token::TokenAccount;

use crate::{error::ErrorCode, state::merkle_distributor::MerkleDistributor};

/// [merkle_distributor::set_clawback_receiver] accounts.
#[derive(Accounts)]
pub struct SetClawbackReceiver<'info> {
    /// The [MerkleDistributor].
    #[account(mut)]
    pub distributor: Account<'info, MerkleDistributor>,

    /// New clawback account
    #[account(token::mint=distributor.mint)]
    pub new_clawback_account: Account<'info, TokenAccount>,

    /// Admin signer
    #[account(mut, address = distributor.admin @ ErrorCode::Unauthorized)]
    pub admin: Signer<'info>,
}

/// Sets new clawback receiver token account
/// CHECK:
///     1. The new clawback receiver is not the same as the old one
#[allow(clippy::result_large_err)]
pub fn handle_set_clawback_receiver(ctx: Context<SetClawbackReceiver>) -> Result<()> {
    require!(
        ctx.accounts.distributor.clawback_receiver.key() != ctx.accounts.new_clawback_account.key(),
        ErrorCode::SameClawbackReceiver
    );

    let distributor = &mut ctx.accounts.distributor;

    let new_clawback_account = *ctx.accounts.new_clawback_account.to_account_info().key;

    distributor.clawback_receiver = new_clawback_account;

    // Note: might get truncated, do not rely on
    msg!(
        "set new clawback receiver ATA to {}, owned by {}",
        new_clawback_account,
        ctx.accounts.new_clawback_account.owner
    );

    Ok(())
}
