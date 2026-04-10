//! Alert System — emits structured Soroban Events for TTL state transitions.

use soroban_sdk::{symbol_short, Env, Symbol};
use crate::types::EntryState;

/// Emits a Soroban Event corresponding to the entry's lifecycle state.
pub struct AlertSystem;

impl AlertSystem {
    /// Emit an event for the given key and state.
    /// No event is emitted for `EntryState::Live` (healthy state).
    pub fn emit(env: &Env, key: &Symbol, state: &EntryState) {
        match state {
            EntryState::Live => {}
            EntryState::Warning => {
                env.events().publish(
                    (symbol_short!("ttl_warn"), key.clone()),
                    key.clone(),
                );
            }
            EntryState::Archived => {
                env.events().publish(
                    (symbol_short!("archived"), key.clone()),
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
}
