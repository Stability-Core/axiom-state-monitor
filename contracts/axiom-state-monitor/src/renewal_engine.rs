//! Renewal Engine — extends TTL for Persistent entries and estimates rent fees.
#![deny(missing_docs)]

use soroban_sdk::{Env, Symbol};

/// Stroops per byte per ledger used to estimate rent fees.
///
/// This is a conservative placeholder. Production deployments should read the
/// actual fee rate from `env.ledger()` fee config once that API is stable.
const FEE_RATE_STROOPS_PER_BYTE_PER_LEDGER: u64 = 1;

/// Handles TTL extension and rent fee estimation for Persistent storage entries.
pub struct RenewalEngine;

impl RenewalEngine {
    /// Extend the TTL of a Persistent storage entry so it stays live for at
    /// least `ledgers` more ledgers.
    ///
    /// Uses `extend_ttl(key, ledgers, ledgers)` which triggers an extension
    /// only when the remaining TTL falls below `ledgers`, making repeated calls
    /// safe and cheap (no unnecessary rent paid).
    ///
    /// The entry must exist in persistent storage; guards against Dead/Archived
    /// states are enforced in the contract layer before this is called.
    pub fn extend(env: &Env, key: &Symbol, ledgers: u32) {
        if env.storage().persistent().has(key) {
            env.storage().persistent().extend_ttl(key, ledgers, ledgers);
        }
    }

    /// Estimate the rent fee in stroops to extend an entry of `entry_size_bytes`
    /// by `ledgers` ledgers.
    ///
    /// Formula: `size_bytes × FEE_RATE × ledgers`
    pub fn calc_fee(_env: &Env, entry_size_bytes: u32, ledgers: u32) -> u64 {
        (entry_size_bytes as u64)
            .saturating_mul(FEE_RATE_STROOPS_PER_BYTE_PER_LEDGER)
            .saturating_mul(ledgers as u64)
    }
}
