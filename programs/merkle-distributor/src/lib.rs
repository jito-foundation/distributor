//! A program for distributing tokens efficiently via uploading a [Merkle root](https://en.wikipedia.org/wiki/Merkle_tree).
//!
//! This program is largely based off of [Uniswap's Merkle Distributor](https://github.com/Uniswap/merkle-distributor).
//!
//! # Rationale
//!
//! Although Solana has low fees for executing transactions, it requires staking tokens to pay for storage costs, also known as "rent". These rent costs can add up when sending tokens to thousands or tens of thousands of wallets, making it economically unreasonable to distribute tokens to everyone.
//!
//! The Merkle distributor, pioneered by [Uniswap](https://github.com/Uniswap/merkle-distributor), solves this issue by deriving a 256-bit "root hash" from a tree of balances. This puts the gas cost on the claimer. Solana has the additional advantage of being able to reclaim rent from closed token accounts, so the net cost to the user should be around `0.000010 SOL` (at the time of writing).
//!
//! The Merkle distributor is also significantly easier to manage from an operations perspective, since one does not need to send a transaction to each individual address that may be redeeming tokens.

#![allow(clippy::too_many_arguments)]
use anchor_lang::prelude::*;
use instructions::*;

pub mod error;
pub mod instructions;
pub mod state;

declare_id!("meRjbQXFNf5En86FXT2YPz1dQzLj4Yb3xK8u1MVgqpb");

#[program]
pub mod merkle_distributor {
    use super::*;

    /// READ THE FOLLOWING:
    ///
    /// This instruction is susceptible to frontrunning that could result in loss of funds if not handled properly.
    ///
    /// An attack could look like:
    /// - A legitimate user opens a new distributor.
    /// - Someone observes the call to this instruction.
    /// - They replace the clawback_receiver, admin, or time parameters with their own.
    ///
    /// One situation that could happen here is the attacker replaces the admin and clawback_receiver with their own
    /// and sets the clawback_start_ts with the minimal time allowed. After clawback_start_ts has elapsed,
    /// the attacker can steal all funds from the distributor to their own clawback_receiver account.
    ///
    /// HOW TO AVOID:
    /// - When you call into this instruction, ensure your transaction succeeds.
    /// - To be extra safe, after your transaction succeeds, read back the state of the created MerkleDistributor account and
    ///   assert the parameters are what you expect, most importantly the clawback_receiver and admin.
    /// - If your transaction fails, double check the value on-chain matches what you expect.
    #[allow(clippy::result_large_err)]
    pub fn new_distributor(
        ctx: Context<NewDistributor>,
        version: u64,
        root: [u8; 32],
        max_total_claim: u64,
        max_num_nodes: u64,
        start_vesting_ts: i64,
        end_vesting_ts: i64,
        clawback_start_ts: i64,
        enable_slot: u64,
        closable: bool,
    ) -> Result<()> {
        handle_new_distributor(
            ctx,
            version,
            root,
            max_total_claim,
            max_num_nodes,
            start_vesting_ts,
            end_vesting_ts,
            clawback_start_ts,
            enable_slot,
            closable,
        )
    }
    /// only available in test phase
    #[allow(clippy::result_large_err)]
    pub fn close_distributor(ctx: Context<CloseDistributor>) -> Result<()> {
        handle_close_distributor(ctx)
    }
    /// only available in test phase
    #[allow(clippy::result_large_err)]
    pub fn close_claim_status(ctx: Context<CloseClaimStatus>) -> Result<()> {
        handle_close_status(ctx)
    }

    #[allow(clippy::result_large_err)]
    pub fn set_enable_slot(ctx: Context<SetEnableSlot>, enable_slot: u64) -> Result<()> {
        handle_set_enable_slot(ctx, enable_slot)
    }

    #[allow(clippy::result_large_err)]
    pub fn new_claim(
        ctx: Context<NewClaim>,
        amount_unlocked: u64,
        amount_locked: u64,
        proof: Vec<[u8; 32]>,
    ) -> Result<()> {
        handle_new_claim(ctx, amount_unlocked, amount_locked, proof)
    }

    #[allow(clippy::result_large_err)]
    pub fn claim_locked(ctx: Context<ClaimLocked>) -> Result<()> {
        handle_claim_locked(ctx)
    }

    #[allow(clippy::result_large_err)]
    pub fn clawback(ctx: Context<Clawback>) -> Result<()> {
        handle_clawback(ctx)
    }

    #[allow(clippy::result_large_err)]
    pub fn set_clawback_receiver(ctx: Context<SetClawbackReceiver>) -> Result<()> {
        handle_set_clawback_receiver(ctx)
    }

    #[allow(clippy::result_large_err)]
    pub fn set_admin(ctx: Context<SetAdmin>) -> Result<()> {
        handle_set_admin(ctx)
    }
}
