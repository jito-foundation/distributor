use anchor_lang::prelude::*;

use crate::error::ErrorCode::ArithmeticError;

/// Holds whether or not a claimant has claimed tokens.
#[account]
#[derive(Default)]
pub struct ClaimStatus {
    /// Authority that claimed the tokens.
    pub claimant: Pubkey,
    /// Locked amount  
    pub locked_amount: u64,
    /// Locked amount withdrawn
    pub locked_amount_withdrawn: u64,
    /// Unlocked amount
    pub unlocked_amount: u64,
    /// indicate that whether admin can close this account, for testing purpose
    pub closable: bool,
    /// admin of merkle tree, store for for testing purpose
    pub admin: Pubkey,
}

impl ClaimStatus {
    pub const LEN: usize = 8 + std::mem::size_of::<ClaimStatus>();

    /// Returns amount withdrawable, factoring in unlocked tokens and previous withdraws.
    /// payout is difference between the amount unlocked and the amount withdrawn
    #[allow(clippy::result_large_err)]
    pub fn amount_withdrawable(&self, curr_ts: i64, start_ts: i64, end_ts: i64) -> Result<u64> {
        let amount = self
            .unlocked_amount(curr_ts, start_ts, end_ts)?
            .checked_sub(self.locked_amount_withdrawn)
            .ok_or(ArithmeticError)?;

        Ok(amount)
    }

    /// Total amount unlocked
    /// Equal to (time_into_unlock / total_unlock_time) * locked_amount  
    /// Multiplication safety:
    ///    The maximum possible product is (2^64 -1) * (2^64 -1) = 2^128 - 2^65 + 1
    ///    which is less than 2^128 - 1 (the maximum value of a u128), meaning that
    ///    the multiplication will never overflow
    /// Truncation from u128 to u64:
    ///     Casting a u128 to a u64 will truncate the 64 higher order bits, which rounds
    ///     down from the user.
    ///     in order to avoid truncation, the final result must be less than 2^64 - 1.
    ///     Rewriting the terms, we get (time_into_unlock * locked_amount) / total_unlock_time < 2^64 - 1
    ///     We know time_into_unlock and total_unlock_time are both approximately the same size, so we can
    ///     approximate the above as:
    ///         b < 2^64 -1.
    ///     Since b is a i64, this is always true, so no truncation can occur
    #[allow(clippy::result_large_err)]
    pub fn unlocked_amount(&self, curr_ts: i64, start_ts: i64, end_ts: i64) -> Result<u64> {
        if curr_ts >= start_ts {
            if curr_ts >= end_ts {
                Ok(self.locked_amount)
            } else {
                let time_into_unlock = curr_ts.checked_sub(start_ts).ok_or(ArithmeticError)?;
                let total_unlock_time = end_ts.checked_sub(start_ts).ok_or(ArithmeticError)?;

                let amount = ((time_into_unlock as u128)
                    .checked_mul(self.locked_amount as u128)
                    .ok_or(ArithmeticError)?)
                .checked_div(total_unlock_time as u128)
                .ok_or(ArithmeticError)? as u64;

                Ok(amount)
            }
        } else {
            Ok(0)
        }
    }
}
