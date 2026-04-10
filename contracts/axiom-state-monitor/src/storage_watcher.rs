//! Storage Watcher — queries `live_until_ledger_seq` and classifies entry state.

use soroban_sdk::{Env, Symbol};
use crate::types::EntryState;

/// Queries Persistent storage TTL and classifies the lifecycle state.
pub struct StorageWatcher<'a> {
    env: &'a Env,
}

impl<'a> StorageWatcher<'a> {
    /// Create a new watcher bound to the current environment.
    pub fn new(env: &'a Env) -> Self {
        Self { env }
    }

    /// Returns the number of ledgers remaining until the entry is archived.
    /// Returns `None` if the entry does not exist (Dead state).
    pub fn get_ttl(&self, key: &Symbol) -> Option<u32> {
        let current = self.env.ledger().sequence();
        // get_ttl returns the live_until_ledger_seq for Persistent entries
        let live_until = self.env.storage().persistent().get_ttl(key)?;
        Some(live_until.saturating_sub(current))
    }

    /// Classify the entry into its lifecycle state given a warning threshold.
    pub fn classify(&self, key: &Symbol, threshold: u32) -> EntryState {
        match self.get_ttl(key) {
            None => EntryState::Dead,
            Some(0) => EntryState::Archived,
            Some(ttl) if ttl <= threshold => EntryState::Warning,
            Some(_) => EntryState::Live,
        }
    }
}
