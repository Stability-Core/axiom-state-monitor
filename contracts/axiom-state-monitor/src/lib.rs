#![no_std]
#![deny(missing_docs)]

//! Axiom State Monitor — Soroban contract for TTL monitoring and renewal
//! of Persistent storage entries on the Stellar network.

use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short,
    Address, Env, Symbol, Vec,
};

mod storage_watcher;
mod renewal_engine;
mod alert_system;
mod types;

pub use types::EntryState;

use storage_watcher::StorageWatcher;
use renewal_engine::RenewalEngine;
use alert_system::AlertSystem;

// Storage keys
const ADMIN_KEY: Symbol = symbol_short!("ADMIN");
const THRESHOLD_KEY: Symbol = symbol_short!("THRESHOLD");
const WATCHED_KEY: Symbol = symbol_short!("WATCHED");

/// Default TTL warning threshold in ledgers (~24 hours at 5s/ledger)
const DEFAULT_THRESHOLD: u32 = 17_280;

#[contract]
pub struct AxiomStateMonitor;

#[contractimpl]
impl AxiomStateMonitor {
    /// Initialize the contract with an admin address and optional TTL threshold.
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
            .set(&WATCHED_KEY, &Vec::<Symbol>::new(&env));
    }

    /// Add a key to the watch list. Admin only.
    pub fn watch(env: Env, key: Symbol) {
        Self::require_admin(&env);
        let mut keys: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&WATCHED_KEY)
            .unwrap_or_else(|| Vec::new(&env));
        keys.push_back(key);
        env.storage().instance().set(&WATCHED_KEY, &keys);
    }

    /// Remove a key from the watch list. Admin only.
    pub fn unwatch(env: Env, key: Symbol) {
        Self::require_admin(&env);
        let keys: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&WATCHED_KEY)
            .unwrap_or_else(|| Vec::new(&env));
        let mut updated = Vec::new(&env);
        for k in keys.iter() {
            if k != key {
                updated.push_back(k);
            }
        }
        env.storage().instance().set(&WATCHED_KEY, &updated);
    }

    /// Query the TTL and state of a single Persistent storage key.
    pub fn check_entry(env: Env, key: Symbol) -> EntryState {
        let threshold: u32 = env
            .storage()
            .instance()
            .get(&THRESHOLD_KEY)
            .unwrap_or(DEFAULT_THRESHOLD);
        let watcher = StorageWatcher::new(&env);
        let state = watcher.classify(&key, threshold);
        AlertSystem::emit(&env, &key, &state);
        state
    }

    /// Run a batch TTL check across all watched keys.
    pub fn check_all(env: Env) {
        let threshold: u32 = env
            .storage()
            .instance()
            .get(&THRESHOLD_KEY)
            .unwrap_or(DEFAULT_THRESHOLD);
        let keys: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&WATCHED_KEY)
            .unwrap_or_else(|| Vec::new(&env));
        let watcher = StorageWatcher::new(&env);
        for key in keys.iter() {
            let state = watcher.classify(&key, threshold);
            AlertSystem::emit(&env, &key, &state);
        }
    }

    /// Extend TTL for a Persistent key by `ledgers` ledgers. Admin only.
    pub fn extend_ttl(env: Env, key: Symbol, ledgers: u32) {
        Self::require_admin(&env);
        RenewalEngine::extend(&env, &key, ledgers);
    }

    /// Calculate the estimated rent fee to extend a key by `ledgers` ledgers.
    pub fn calc_fee(env: Env, entry_size_bytes: u32, ledgers: u32) -> u64 {
        RenewalEngine::calc_fee(&env, entry_size_bytes, ledgers)
    }

    /// Update the TTL warning threshold. Admin only.
    pub fn set_threshold(env: Env, threshold: u32) {
        Self::require_admin(&env);
        env.storage().instance().set(&THRESHOLD_KEY, &threshold);
    }

    // --- Internal ---

    fn require_admin(env: &Env) {
        let admin: Address = env
            .storage()
            .instance()
            .get(&ADMIN_KEY)
            .expect("not initialized");
        admin.require_auth();
    }
}
