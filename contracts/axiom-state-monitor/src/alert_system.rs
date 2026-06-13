//! Alert System — emits structured Soroban Events for TTL state transitions.
#![deny(missing_docs)]

use soroban_sdk::{symbol_short, Env, Symbol};
use crate::types::EntryState;

/// Emits a Soroban Event corresponding to the entry's lifecycle state.
pub struct AlertSystem;

impl AlertSystem {
    /// Emit an event for the given key and state.
    ///
    /// Events emitted:
    /// - `ttl_warning`    — TTL is at or below the warning threshold (REQ-004)
    /// - `ttl_critical`   — TTL has reached zero; entry is archived (REQ-005)
    /// - `dead`           — Entry is permanently deleted
    ///
    /// No event is emitted for `EntryState::Live`.
    pub fn emit(env: &Env, key: &Symbol, state: &EntryState) {
        match state {
            EntryState::Live => {}
            EntryState::Warning => {
                env.events().publish(
                    (Symbol::new(env, "ttl_warning"), key.clone()),
                    key.clone(),
                );
            }
            EntryState::Archived => {
                env.events().publish(
                    (Symbol::new(env, "ttl_critical"), key.clone()),
                    key.clone(),
                );
            }
            EntryState::Dead => {
                env.events().publish(
                    (symbol_short!("dead"), key.clone()),
                    key.clone(),
                );
            }
        }
    }

    /// Emit an `archived_entry` event signalling off-chain tooling that a
    /// `RestoreFootprint` transaction is required before TTL can be extended.
    /// Satisfies REQ-008.
    pub fn emit_archived_entry(env: &Env, key: &Symbol) {
        env.events().publish(
            (Symbol::new(env, "archived_entry"), key.clone()),
            key.clone(),
        );
    }
}
