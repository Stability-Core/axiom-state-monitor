//! Contract error codes for the Axiom State Monitor.
// The #[contracterror] macro generates conversion impls that cannot carry docs.
#![allow(missing_docs)]

use soroban_sdk::contracterror;

/// Errors that the Axiom State Monitor contract can return.
#[allow(missing_docs)]
#[contracterror]
#[derive(Copy, Clone, Debug, PartialEq)]
pub enum ContractError {
    /// `initialize` was called after the contract was already set up.
    AlreadyInitialized = 1,
    /// A mutating function was called before `initialize`.
    NotInitialized = 2,
    /// `extend_ttl` was called on an entry whose TTL has already expired.
    /// Submit a `RestoreFootprint` transaction before retrying.
    EntryArchived = 3,
    /// `extend_ttl` was called on a key that is not in the watch registry.
    EntryDead = 4,
    /// A threshold value of zero is not permitted.
    InvalidThreshold = 5,
}
