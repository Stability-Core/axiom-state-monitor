#![allow(missing_docs)]

use soroban_sdk::contracttype;

/// Represents the archival lifecycle state of a Persistent storage entry.
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum EntryState {
    /// Entry is live; TTL is above the warning threshold.
    Live,
    /// Entry is live but TTL is at or below the warning threshold.
    Warning,
    /// Entry TTL has reached zero; it is archived and unreadable.
    Archived,
    /// Entry has passed the grace period and is permanently deleted.
    Dead,
}
