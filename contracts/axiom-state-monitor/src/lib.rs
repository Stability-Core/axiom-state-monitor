#![no_std]
// missing_docs is enforced in individual modules; soroban proc-macros (#[contract],
// #[contractimpl]) emit undocumented items that cannot be attributed, so we
// cannot deny at the crate root without breaking compilation.
#![warn(missing_docs)]

//! Axiom State Monitor — Soroban smart contract for monitoring and renewing
//! the TTL of Persistent storage entries on the Stellar network.
//!
//! Prevents State Archival data loss by watching critical storage keys and
//! emitting structured Soroban Events before TTL expiry.
//!
//! ## How TTL tracking works
//!
//! Soroban-sdk 20.x does not expose per-entry TTL to contracts at runtime.
//! Axiom State Monitor therefore maintains an internal registry:
//!
//! ```text
//! WATCHED_KEY  →  Map<Symbol, u32>   (key → live_until_ledger_seq)
//! ```
//!
//! Callers provide the current `live_until_ledger_seq` when registering a key
//! via `watch`. The registry is updated whenever `extend_ttl` is called through
//! this contract. Use `update_live_until` to sync the registry after an
//! out-of-band extension.

use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, Map, Symbol};

mod alert_system;
mod error;
mod renewal_engine;
mod storage_watcher;
mod types;

#[cfg(test)]
mod tests;

pub use error::ContractError;
pub use types::EntryState;

use alert_system::AlertSystem;
use renewal_engine::RenewalEngine;
use storage_watcher::StorageWatcher;

// ── Instance storage keys ────────────────────────────────────────────────────

const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const THRESHOLD_KEY: Symbol = symbol_short!("THRESHOLD");
/// Maps `Symbol → u32` (watched key → live_until_ledger_seq).
const WATCHED_KEY: Symbol = symbol_short!("WATCHED");

/// Default TTL warning threshold in ledgers (~24 h at 5 s/ledger).
const DEFAULT_THRESHOLD: u32 = 17_280;

// ── Contract ─────────────────────────────────────────────────────────────────

/// The Axiom State Monitor Soroban contract.
// #[contract] generates an undocumented inner field; #[contractimpl] generates
// a client struct whose methods cannot carry hand-written docs.
#[allow(missing_docs)]
#[contract]
pub struct AxiomStateMonitor;

#[allow(missing_docs)]
#[contractimpl]
impl AxiomStateMonitor {
    // ── Initialisation ───────────────────────────────────────────────────────

    /// One-time setup: set the admin address and optional TTL warning threshold.
    ///
    /// Panics if the contract is already initialised.
    pub fn initialize(env: Env, admin: Address, threshold: Option<u32>) {
        if env.storage().instance().has(&ADMIN_KEY) {
            panic!("already initialized");
        }
        env.storage().instance().set(&ADMIN_KEY, &admin);
        env.storage()
            .instance()
            .set(&THRESHOLD_KEY, &threshold.unwrap_or(DEFAULT_THRESHOLD));
        env.storage()
            .instance()
            .set(&WATCHED_KEY, &Map::<Symbol, u32>::new(&env));
    }

    // ── Watch list management ────────────────────────────────────────────────

    /// Add `key` to the watch registry with its current `live_until_ledger_seq`.
    ///
    /// The caller must supply the absolute ledger number at which the entry
    /// expires (obtainable off-chain via Horizon or stellar-cli). Admin only.
    pub fn watch(env: Env, key: Symbol, live_until: u32) {
        Self::require_admin(&env);
        let mut map = Self::watched_map(&env);
        map.set(key, live_until);
        env.storage().instance().set(&WATCHED_KEY, &map);
    }

    /// Remove `key` from the watch registry. Admin only.
    pub fn unwatch(env: Env, key: Symbol) {
        Self::require_admin(&env);
        let mut map = Self::watched_map(&env);
        map.remove(key);
        env.storage().instance().set(&WATCHED_KEY, &map);
    }

    /// Update the recorded `live_until_ledger_seq` for `key` after an
    /// out-of-band TTL extension. Admin only.
    pub fn update_live_until(env: Env, key: Symbol, live_until: u32) {
        Self::require_admin(&env);
        let mut map = Self::watched_map(&env);
        map.set(key, live_until);
        env.storage().instance().set(&WATCHED_KEY, &map);
    }

    // ── TTL queries ──────────────────────────────────────────────────────────

    /// Query the TTL and lifecycle state of a single watched key.
    ///
    /// Returns `EntryState::Dead` when the key is not in the watch registry.
    /// Emits `ttl_warning`, `ttl_critical`, or `dead` events as appropriate.
    pub fn check_entry(env: Env, key: Symbol) -> EntryState {
        let state = Self::classify_key(&env, &key);
        AlertSystem::emit(&env, &key, &state);
        state
    }

    /// Run a batch TTL check across every key in the watch registry.
    ///
    /// Emits events for each key that is not in the `Live` state.
    pub fn check_all(env: Env) {
        let map = Self::watched_map(&env);
        for (key, _) in map.iter() {
            let state = Self::classify_key(&env, &key);
            AlertSystem::emit(&env, &key, &state);
        }
    }

    // ── Renewal ──────────────────────────────────────────────────────────────

    /// Extend the TTL of a registered Persistent key by `ledgers` ledgers. Admin only.
    ///
    /// Also updates the internal registry so future `check_entry` calls reflect
    /// the new expiry.
    ///
    /// # Errors
    /// - [`ContractError::EntryDead`] — key is not in the watch registry.
    /// - [`ContractError::EntryArchived`] — recorded `live_until` has already passed.
    ///   Submit a `RestoreFootprint` transaction off-chain, then call
    ///   `update_live_until` before retrying.
    pub fn extend_ttl(env: Env, key: Symbol, ledgers: u32) -> Result<(), ContractError> {
        Self::require_admin(&env);

        match Self::classify_key(&env, &key) {
            EntryState::Dead => return Err(ContractError::EntryDead),
            EntryState::Archived => return Err(ContractError::EntryArchived),
            _ => {}
        }

        RenewalEngine::extend(&env, &key, ledgers);

        // Update registry to reflect the new live_until
        let new_live_until = env.ledger().sequence().saturating_add(ledgers);
        let mut map = Self::watched_map(&env);
        map.set(key, new_live_until);
        env.storage().instance().set(&WATCHED_KEY, &map);

        Ok(())
    }

    /// Signal that an archived or dead entry needs off-chain attention. Admin only.
    ///
    /// - `Archived` → emits `archived_entry` (signals a `RestoreFootprint` is needed)
    /// - `Dead`     → emits `dead`
    /// - `Live` / `Warning` → no event; returns current state
    ///
    /// This function does NOT extend TTL — call `extend_ttl` after the entry
    /// has been restored on-chain.
    pub fn restore_entry(env: Env, key: Symbol) -> EntryState {
        Self::require_admin(&env);
        let state = Self::classify_key(&env, &key);
        match &state {
            EntryState::Archived => AlertSystem::emit_archived_entry(&env, &key),
            EntryState::Dead => AlertSystem::emit(&env, &key, &state),
            _ => {}
        }
        state
    }

    // ── Configuration ────────────────────────────────────────────────────────

    /// Estimate the rent fee in stroops to extend an entry of `entry_size_bytes`
    /// by `ledgers` ledgers.
    pub fn calc_fee(env: Env, entry_size_bytes: u32, ledgers: u32) -> u64 {
        RenewalEngine::calc_fee(&env, entry_size_bytes, ledgers)
    }

    /// Update the TTL warning threshold. Admin only.
    ///
    /// Returns [`ContractError::InvalidThreshold`] if `threshold` is zero.
    pub fn set_threshold(env: Env, threshold: u32) -> Result<(), ContractError> {
        Self::require_admin(&env);
        if threshold == 0 {
            return Err(ContractError::InvalidThreshold);
        }
        env.storage().instance().set(&THRESHOLD_KEY, &threshold);
        Ok(())
    }

    // ── Internal helpers ─────────────────────────────────────────────────────

    fn require_admin(env: &Env) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN_KEY)
            .expect("not initialized");
        admin.require_auth();
    }

    fn threshold(env: &Env) -> u32 {
        env.storage()
            .instance()
            .get(&THRESHOLD_KEY)
            .unwrap_or(DEFAULT_THRESHOLD)
    }

    fn watched_map(env: &Env) -> Map<Symbol, u32> {
        env.storage()
            .instance()
            .get(&WATCHED_KEY)
            .unwrap_or_else(|| Map::new(env))
    }

    fn classify_key(env: &Env, key: &Symbol) -> EntryState {
        let map = Self::watched_map(env);
        match map.get(key.clone()) {
            None => EntryState::Dead,
            Some(live_until) => {
                let threshold = Self::threshold(env);
                StorageWatcher::new(env).classify(live_until, threshold)
            }
        }
    }
}
