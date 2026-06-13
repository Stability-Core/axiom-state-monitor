//! Storage Watcher — classifies entry lifecycle state from a known `live_until` value.
#![deny(missing_docs)]

use soroban_sdk::Env;
use crate::types::EntryState;

/// Classifies Persistent storage entries based on their recorded expiry ledger.
///
/// In soroban-sdk 20.x the host does not expose per-entry TTL to contracts, so
/// this module works against `live_until_ledger_seq` values that are tracked
/// by the contract's internal registry (`WATCHED_KEY` map in `lib.rs`).
pub struct StorageWatcher<'a> {
    env: &'a Env,
}

impl<'a> StorageWatcher<'a> {
    /// Create a new watcher bound to the current environment.
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }

    /// Classify an entry given its absolute `live_until_ledger_seq` and the
    /// configured warning threshold (in ledgers).
    ///
    /// | Remaining TTL          | State     |
    /// |------------------------|-----------|
    /// | `0` (expired)          | Archived  |
    /// | `> 0` and `≤ threshold`| Warning   |
    /// | `> threshold`          | Live      |
    pub fn classify(&self, live_until: u32, threshold: u32) -> EntryState {
        let current = self.env.ledger().sequence();
        let remaining = live_until.saturating_sub(current);
        match remaining {
            0 => EntryState::Archived,
            r if r <= threshold => EntryState::Warning,
            _ => EntryState::Live,
        }
    }
}
