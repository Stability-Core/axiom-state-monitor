#![cfg(test)]

extern crate std;

use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Events, Ledger, LedgerInfo},
    Address, Env, Symbol, TryIntoVal,
};

use crate::{AxiomStateMonitor, AxiomStateMonitorClient, ContractError, EntryState};

// ── Test helpers ─────────────────────────────────────────────────────────────

fn make_env(sequence: u32) -> Env {
    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set(LedgerInfo {
        timestamp: 0,
        protocol_version: 20,
        sequence_number: sequence,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 4096,
        max_entry_ttl: 3_110_400, // ~1 year
    });
    env
}

fn setup(env: &Env, threshold: Option<u32>) -> (AxiomStateMonitorClient, Address) {
    let contract_id = env.register_contract(None, AxiomStateMonitor);
    let client = AxiomStateMonitorClient::new(env, &contract_id);
    let admin = Address::generate(env);
    client.initialize(&admin, &threshold);
    (client, admin)
}

// ── WAVE-006: classify → Live ─────────────────────────────────────────────────

#[test]
fn test_classify_live() {
    // threshold = 17_280; TTL remaining = 20_000 > threshold → Live
    let env = make_env(1_000);
    let (client, _) = setup(&env, None); // default threshold 17_280
    let key = symbol_short!("DATA");

    // live_until = 1_000 + 20_000 = 21_000
    client.watch(&key, &21_000);

    assert_eq!(client.check_entry(&key), EntryState::Live);
}

// ── WAVE-007: classify → Archived when ledger >= live_until ──────────────────

#[test]
fn test_classify_archived_at_expiry() {
    let env = make_env(1_000);
    let (client, _) = setup(&env, Some(500));
    let key = symbol_short!("EXPKEY");

    // live_until = 2_000; entry expires at ledger 2_000
    client.watch(&key, &2_000);

    // Advance to exactly live_until: remaining = 2_000 - 2_000 = 0 → Archived
    env.ledger().set(LedgerInfo {
        sequence_number: 2_000,
        ..env.ledger().get()
    });

    assert_eq!(client.check_entry(&key), EntryState::Archived);
}

// ── WAVE-008: get_ttl returns correct delta ───────────────────────────────────

#[test]
fn test_ttl_delta_transitions() {
    let env = make_env(1_000);
    let (client, _) = setup(&env, Some(3_000));
    let key = symbol_short!("K");

    // TTL remaining at ledger 1_000 = 6_000 - 1_000 = 5_000 > threshold 3_000 → Live
    client.watch(&key, &6_000);
    assert_eq!(client.check_entry(&key), EntryState::Live);

    // Advance to 3_500 → remaining = 6_000 - 3_500 = 2_500 ≤ threshold 3_000 → Warning
    env.ledger().set(LedgerInfo {
        sequence_number: 3_500,
        ..env.ledger().get()
    });
    assert_eq!(client.check_entry(&key), EntryState::Warning);
}

// ── WAVE-009: ttl_warning event emitted when TTL ≤ threshold ─────────────────

#[test]
fn test_ttl_warning_event_emitted() {
    let env = make_env(1_000);
    let (client, _) = setup(&env, Some(3_000));
    let key = symbol_short!("EVT");

    client.watch(&key, &6_000);

    // Advance so remaining TTL (2_000) < threshold (3_000)
    env.ledger().set(LedgerInfo {
        sequence_number: 4_000,
        ..env.ledger().get()
    });

    client.check_entry(&key);

    let warning_sym = Symbol::new(&env, "ttl_warning");
    let found = env.events().all().iter().any(|e| {
        let topics: soroban_sdk::Vec<soroban_sdk::Val> = e.1;
        topics
            .get(0)
            .and_then(|v| v.try_into_val(&env).ok())
            .map(|s: Symbol| s == warning_sym)
            .unwrap_or(false)
    });
    assert!(found, "expected ttl_warning event");
}

// ── WAVE-010: ttl_critical event emitted when TTL == 0 ───────────────────────

#[test]
fn test_ttl_critical_event_emitted() {
    let env = make_env(1_000);
    let (client, _) = setup(&env, Some(500));
    let key = symbol_short!("CRIT");

    client.watch(&key, &3_000);

    // Advance past expiry
    env.ledger().set(LedgerInfo {
        sequence_number: 3_001,
        ..env.ledger().get()
    });

    client.check_entry(&key);

    let critical_sym = Symbol::new(&env, "ttl_critical");
    let found = env.events().all().iter().any(|e| {
        let topics: soroban_sdk::Vec<soroban_sdk::Val> = e.1;
        topics
            .get(0)
            .and_then(|v| v.try_into_val(&env).ok())
            .map(|s: Symbol| s == critical_sym)
            .unwrap_or(false)
    });
    assert!(found, "expected ttl_critical event");
}

// ── WAVE-011: renewal blocked for Archived entries ────────────────────────────

#[test]
fn test_renewal_blocked_for_archived() {
    let env = make_env(1_000);
    let (client, _) = setup(&env, Some(500));
    let key = symbol_short!("ARC");

    client.watch(&key, &3_000);

    // Advance past expiry
    env.ledger().set(LedgerInfo {
        sequence_number: 3_001,
        ..env.ledger().get()
    });

    assert_eq!(
        client.try_extend_ttl(&key, &10_000),
        Err(Ok(ContractError::EntryArchived)),
    );
}

// ── WAVE-012: renewal blocked for Dead (unregistered) entries ────────────────

#[test]
fn test_renewal_blocked_for_dead() {
    let env = make_env(1_000);
    let (client, _) = setup(&env, None);
    let key = symbol_short!("GHOST"); // never registered

    assert_eq!(
        client.try_extend_ttl(&key, &10_000),
        Err(Ok(ContractError::EntryDead)),
    );
}

// ── WAVE-013: fee calculation matches size × rate × ledgers ──────────────────

#[test]
fn test_fee_calculation() {
    let env = make_env(1_000);
    let (client, _) = setup(&env, None);

    // FEE_RATE = 1 stroop/byte/ledger
    assert_eq!(client.calc_fee(&1024, &100_000), 1024u64 * 100_000u64);
    assert_eq!(client.calc_fee(&0, &100_000), 0);
    assert_eq!(client.calc_fee(&512, &0), 0);
}

// ── Extra: Dead state — key not registered ───────────────────────────────────

#[test]
fn test_classify_dead_when_key_absent() {
    let env = make_env(1_000);
    let (client, _) = setup(&env, None);
    let key = symbol_short!("NONE");

    assert_eq!(client.check_entry(&key), EntryState::Dead);
}

// ── Extra: Warning band ───────────────────────────────────────────────────────

#[test]
fn test_classify_warning() {
    // remaining = 6_000 - 1_000 = 5_000; threshold = 10_000 → Warning
    let env = make_env(1_000);
    let (client, _) = setup(&env, Some(10_000));
    let key = symbol_short!("WARN");

    client.watch(&key, &6_000);

    assert_eq!(client.check_entry(&key), EntryState::Warning);
}

// ── Extra: watch / unwatch / check_all round-trip ────────────────────────────

#[test]
fn test_watch_unwatch_check_all() {
    let env = make_env(1_000);
    let (client, _) = setup(&env, None);
    let key = symbol_short!("WK");

    client.watch(&key, &21_000);
    client.check_all(); // should not panic

    client.unwatch(&key);
    client.check_all(); // empty registry — also fine
}

// ── Extra: extend_ttl updates the registry live_until ────────────────────────

#[test]
fn test_extend_ttl_updates_registry() {
    let env = make_env(1_000);
    let (client, _) = setup(&env, Some(3_000));
    let key = symbol_short!("EXT");

    // Warning: remaining = 4_000 - 1_000 = 3_000 == threshold → Warning
    client.watch(&key, &4_000);
    assert_eq!(client.check_entry(&key), EntryState::Warning);

    // Extend by 20_000 → new live_until = 1_000 + 20_000 = 21_000
    client.extend_ttl(&key, &20_000);

    // Now Live: remaining = 21_000 - 1_000 = 20_000 > threshold 3_000
    assert_eq!(client.check_entry(&key), EntryState::Live);
}

// ── Extra: double initialize panics ──────────────────────────────────────────

#[test]
#[should_panic(expected = "already initialized")]
fn test_double_initialize_panics() {
    let env = make_env(1_000);
    let (client, admin) = setup(&env, None);
    client.initialize(&admin, &None);
}

// ── Extra: set_threshold rejects zero ────────────────────────────────────────

#[test]
fn test_set_threshold_rejects_zero() {
    let env = make_env(1_000);
    let (client, _) = setup(&env, None);
    assert_eq!(
        client.try_set_threshold(&0),
        Err(Ok(ContractError::InvalidThreshold)),
    );
}

// ── Extra: restore_entry emits archived_entry event ──────────────────────────

#[test]
fn test_restore_entry_emits_event_for_archived() {
    let env = make_env(1_000);
    let (client, _) = setup(&env, Some(500));
    let key = symbol_short!("REST");

    client.watch(&key, &3_000);

    // Advance past expiry
    env.ledger().set(LedgerInfo {
        sequence_number: 3_001,
        ..env.ledger().get()
    });

    let state = client.restore_entry(&key);
    assert_eq!(state, EntryState::Archived);

    let arch_sym = Symbol::new(&env, "archived_entry");
    let found = env.events().all().iter().any(|e| {
        let topics: soroban_sdk::Vec<soroban_sdk::Val> = e.1;
        topics
            .get(0)
            .and_then(|v| v.try_into_val(&env).ok())
            .map(|s: Symbol| s == arch_sym)
            .unwrap_or(false)
    });
    assert!(found, "expected archived_entry event from restore_entry");
}

// ── Extra: update_live_until allows correcting the registry ──────────────────

#[test]
fn test_update_live_until() {
    let env = make_env(1_000);
    let (client, _) = setup(&env, Some(3_000));
    let key = symbol_short!("UPD");

    // Register as expired
    client.watch(&key, &500); // 500 < 1_000 → Archived
    assert_eq!(client.check_entry(&key), EntryState::Archived);

    // Sync after off-chain restoration
    client.update_live_until(&key, &30_000);
    assert_eq!(client.check_entry(&key), EntryState::Live);
}
