use anchor_lang::{account, context::Context, prelude::*, Accounts, Key, ToAccountInfo};

use crate::{error::ErrorCode, state::claim_status::ClaimStatus};

// Accounts for [merkle_distributor::close_claim_status].
#[derive(Accounts)]
pub struct CloseClaimStatus<'info> {
    #[account(
        mut,
        has_one = claimant,
        has_one = admin,
        constraint = claim_status.closable @ ErrorCode::CannotCloseClaimStatus,
        close = claimant,
    )]
    pub claim_status: Account<'info, ClaimStatus>,

    /// CHECK: claimant
    #[account(mut)]
    pub claimant: UncheckedAccount<'info>,

    pub admin: Signer<'info>,
}

#[allow(clippy::result_large_err)]
pub fn handle_close_status(ctx: Context<CloseClaimStatus>) -> Result<()> {
    Ok(())
}
