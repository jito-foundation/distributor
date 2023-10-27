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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normal_unlocking_scenario() {
        let claim_status = ClaimStatus {
            claimant: Pubkey::new_unique(),
            locked_amount: 100,
            unlocked_amount: 0,
            locked_amount_withdrawn: 0,
        };
        let curr_ts = 50;
        let start_ts = 0;
        let end_ts = 100;
        assert_eq!(
            claim_status.unlocked_amount(curr_ts, start_ts, end_ts),
            Ok(50)
        );
    }

    #[test]
    fn test_proportional_unlocking() {
        let claim_status = ClaimStatus {
            claimant: Pubkey::new_unique(),
            locked_amount: 100,
            locked_amount_withdrawn: 0,
            unlocked_amount: 0,
        };
        let start_ts = 0;
        let end_ts = 100;

        assert_eq!(claim_status.unlocked_amount(0, start_ts, end_ts), Ok(0));
        assert_eq!(claim_status.unlocked_amount(25, start_ts, end_ts), Ok(25));
        assert_eq!(claim_status.unlocked_amount(50, start_ts, end_ts), Ok(50));
        assert_eq!(claim_status.unlocked_amount(75, start_ts, end_ts), Ok(75));
        assert_eq!(claim_status.unlocked_amount(100, start_ts, end_ts), Ok(100));
    }

    #[test]
    fn test_unlocked_amount_no_truncation() {
        // Test that even with the maximum possible values for curr_ts, start_ts, end_ts, and locked_amount,
        // the unlocked_amount function will not truncate or overflow

        let locked_amount = u64::MAX;

        // Create a ClaimStatus instance
        let claim_status = ClaimStatus {
            claimant: Pubkey::new_unique(),
            locked_amount,
            unlocked_amount: 0,
            locked_amount_withdrawn: 0,
        };

        // Use large values for time_into_unlock and total_unlock_time, but ensure they are within i64 range
        let start_ts = 0;
        let end_ts = i64::MAX;
        for curr_ts in [0, (end_ts - start_ts) / 2, end_ts] {
            // Calculate the expected amount without risking overflow or truncation
            let time_into_unlock = (curr_ts - start_ts) as u128;
            let total_unlock_time = (end_ts - start_ts) as u128;
            let expected_amount = (time_into_unlock * locked_amount as u128) / total_unlock_time;

            // Perform the calculation using the function
            let calculated_amount = claim_status
                .unlocked_amount(curr_ts, start_ts, end_ts)
                .unwrap();

            // Assert that the calculated amount matches the expected amount and is within u64 bounds
            assert_eq!(calculated_amount as u128, expected_amount);
            assert!(expected_amount <= u64::MAX as u128); // Ensure no truncation would occur
        }
    }

    #[test]
    fn test_unlocking_after_end_time() {
        let claim_status = ClaimStatus {
            claimant: Pubkey::new_unique(),
            locked_amount: 100,
            unlocked_amount: 0,
            locked_amount_withdrawn: 0,
        };
        let curr_ts = 150;
        let start_ts = 0;
        let end_ts = 100;
        assert_eq!(
            claim_status.unlocked_amount(curr_ts, start_ts, end_ts),
            Ok(100)
        );
    }

    #[test]
    fn test_division_by_zero() {
        let claim_status = ClaimStatus {
            claimant: Pubkey::new_unique(),
            locked_amount: 100,
            unlocked_amount: 0,
            locked_amount_withdrawn: 0,
        };
        let curr_ts = 50;
        let start_ts = 100;
        let end_ts = 100;
        assert_eq!(
            claim_status.unlocked_amount(curr_ts, start_ts, end_ts),
            Ok(0)
        );
    }

    #[test]
    fn test_start_greater_than_end() {
        let claim_status = ClaimStatus {
            locked_amount: 100,
            ..Default::default()
        };
        let start_ts = 100;
        let end_ts = 50;

        assert_eq!(claim_status.unlocked_amount(75, start_ts, end_ts), Ok(0));
    }

    #[test]
    fn test_partial_withdraw() {
        for (curr_ts, expected, locked_amount_withdrawn) in [
            (0, 0, 0),     // nothing vested, nothing to withdraw, nothing withdrawable
            (10, 0, 10),   // 1/10th vested, 1/10th withdrawn, nothing withdrawable
            (20, 0, 20),   // 2/10th vested, 2/10th withdrawn, nothing withdrawable
            (50, 0, 50),   // 5/10th vested, 5/10th withdrawn, nothing withdrawable
            (50, 25, 25),  // 5/10th vested, 2.5/10th withdrawn, 25 withdrawable
            (70, 10, 60),  // 7/10th vested, 6/10th withdrawn, 10 withdrawable
            (100, 90, 10), // 10/10th vested, 9/10th withdrawn, 10 withdrawable
            (100, 0, 100), // 10/10th vested, 10/10th withdrawn, nothing withdrawable
        ] {
            let claim_status = ClaimStatus {
                claimant: Pubkey::new_unique(),
                locked_amount: 100,
                unlocked_amount: 0,
                locked_amount_withdrawn,
            };

            assert_eq!(
                claim_status.amount_withdrawable(curr_ts, 0, 100),
                Ok(expected)
            );
        }
    }
}
