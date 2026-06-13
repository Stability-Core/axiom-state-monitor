//! Shared types used across the Axiom State Monitor contract.
// The #[contracttype] macro generates conversion impls that cannot carry docs.
#![allow(missing_docs)]

use soroban_sdk::contracttype;

/// Lifecycle state of a Persistent storage entry.
///
/// The four states form a one-way progression:
///
/// ```text
/// Live ──(TTL ≤ threshold)──► Warning ──(TTL = 0)──► Archived ──(grace ends)──► Dead
///  ▲                                                       │
///  └───────────────(RestoreFootprint + extend_ttl)─────────┘
/// ```
#[allow(missing_docs)]
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum EntryState {
    /// Entry is live; remaining TTL is above the configured warning threshold.
    Live,
    /// Entry is live but remaining TTL is at or below the warning threshold.
    Warning,
    /// Entry TTL has reached zero. It is archived and unreadable until a
    /// `RestoreFootprint` transaction is submitted.
    Archived,
    /// Entry has passed the grace period and is permanently deleted.
    /// It cannot be restored.
    Dead,
}
