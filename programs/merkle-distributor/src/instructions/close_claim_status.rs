use anchor_lang::{account, context::Context, prelude::*, Accounts, Key, ToAccountInfo};
use solana_program::pubkey;

use crate::{error::ErrorCode, state::claim_status::ClaimStatus};

pub fn assert_eq_admin(sender: Pubkey) -> bool {
    let admin = pubkey!("DHLXnJdACTY83yKwnUkeoDjqi4QBbsYGa1v8tJL76ViX");
    sender == admin
}

// Accounts for [merkle_distributor::close_claim_status].
#[derive(Accounts)]
pub struct CloseClaimStatus<'info> {
    #[account(
        mut,
        has_one = claimant,
        close = claimant,
    )]
    pub claim_status: Account<'info, ClaimStatus>,

    /// CHECK: claimant
    #[account(mut)]
    pub claimant: UncheckedAccount<'info>,

    #[account(constraint = assert_eq_admin(admin.key()) @ ErrorCode::Unauthorized)]
    pub admin: Signer<'info>,
}

#[allow(clippy::result_large_err)]
#[cfg(feature = "test")]
pub fn handle_close_status(ctx: Context<CloseClaimStatus>) -> Result<()> {
    Ok(())
}

#[allow(clippy::result_large_err)]
#[cfg(not(feature = "test"))]
pub fn handle_close_status(ctx: Context<CloseClaimStatus>) -> Result<()> {
    return Err(ErrorCode::CannotCloseClaimStatus.into());
}
