use anchor_lang::{account, context::Context, prelude::*, Accounts, Key, ToAccountInfo};
use anchor_spl::token::{self, Token, TokenAccount};

use crate::{error::ErrorCode, state::merkle_distributor::MerkleDistributor};

// Accounts for [merkle_distributor::close_distributor].
#[derive(Accounts)]
pub struct CloseDistributor<'info> {
    /// [MerkleDistributor].
    #[account(
        mut,
        has_one = admin,
        has_one = token_vault,
        constraint = distributor.closable @ ErrorCode::CannotCloseDistributor,
        close = admin
    )]
    pub distributor: Account<'info, MerkleDistributor>,

    /// Clawback receiver token account
    #[account(mut)]
    pub token_vault: Account<'info, TokenAccount>,

    /// Admin wallet, responsible for creating the distributor and paying for the transaction.
    /// Also has the authority to set the clawback receiver and change itself.
    #[account(mut)]
    pub admin: Signer<'info>,

    /// account receive token back
    #[account(mut)]
    pub destination_token_account: Account<'info, TokenAccount>,

    /// The [Token] program.
    pub token_program: Program<'info, Token>,
}

#[allow(clippy::result_large_err)]
pub fn handle_close_distributor(ctx: Context<CloseDistributor>) -> Result<()> {
    let distributor = &ctx.accounts.distributor;
    let seeds = [
        b"MerkleDistributor".as_ref(),
        &distributor.mint.to_bytes(),
        &distributor.version.to_le_bytes(),
        &[ctx.accounts.distributor.bump],
    ];

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            token::Transfer {
                from: ctx.accounts.token_vault.to_account_info(),
                to: ctx.accounts.destination_token_account.to_account_info(),
                authority: ctx.accounts.distributor.to_account_info(),
            },
        )
        .with_signer(&[&seeds[..]]),
        ctx.accounts.token_vault.amount,
    )?;
    Ok(())
}
