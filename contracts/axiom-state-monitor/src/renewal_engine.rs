//! Renewal Engine — calculates rent fees and extends TTL for Persistent entries.

use soroban_sdk::{Env, Symbol};

/// Fee rate constant: stroops per byte per ledger.
/// This is a placeholder — production should read from `env.ledger()` fee config.
const FEE_RATE_STROOPS_PER_BYTE_PER_LEDGER: u64 = 1;

/// Handles TTL extension and rent fee estimation.
pub struct RenewalEngine;

impl RenewalEngine {
    /// Extend the TTL of a Persistent storage entry by `ledgers` ledgers.
    /// Panics if the entry is Archived or Dead (key not present).
    pub fn extend(env: &Env, key: &Symbol, ledgers: u32) {
        // extend_ttl(key, threshold_to_keep, extend_to)
        // We extend from current TTL up to current + ledgers
        let current = env.ledger().sequence();
        let target = current.saturating_add(ledgers);
        env.storage().persistent().extend_ttl(key, ledgers, target);
    }

    /// Estimate the rent fee in stroops to extend an entry of `entry_size_bytes`
    /// by `ledgers` ledgers.
    pub fn calc_fee(_env: &Env, entry_size_bytes: u32, ledgers: u32) -> u64 {
        (entry_size_bytes as u64)
            .saturating_mul(FEE_RATE_STROOPS_PER_BYTE_PER_LEDGER)
            .saturating_mul(ledgers as u64)
    }
}
