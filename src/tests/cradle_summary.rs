//! Tests for the CradleSummary types and their `From` constructors.
//!
//! These unit tests exercise the summary builders in isolation — the
//! glue that reads cached cradle state and feeds it into summary
//! structs. Full-cradle integration tests that drive handshake / move
//! / unroll / on-chain transitions live under `sim-tests` (they need
//! the Simulator harness) and will be added in a follow-up.

use crate::channel_handler::types::UnrollCoin;
use crate::cradle_summary::{CradleSummary, PendingProposalSummary, UnrollCoinSummary};

#[test]
fn unroll_coin_summary_from_default_has_no_conditions_hash() {
    let unroll = UnrollCoin::default();
    let summary = UnrollCoinSummary::from_unroll(&unroll);
    assert_eq!(summary.state_number, 0);
    assert!(
        summary.conditions_hash.is_none(),
        "a freshly-default UnrollCoin has no outcome yet"
    );
}

#[test]
fn cradle_summary_serializes_to_json_shape() {
    // Round-trip a hand-built summary through serde_json so we
    // detect any derive mismatch or non-serializable field
    // regression without standing up a full cradle.
    let summary = CradleSummary {
        channel: None,
        state_number: Some(3),
        signed_state_number: Some(2),
        have_potato: true,
        unroll_coin: Some(UnrollCoinSummary {
            state_number: 3,
            conditions_hash: Some("ab".repeat(32)),
        }),
        pending_proposals: vec![PendingProposalSummary {
            game_id: 7,
            my_contribution: 11,
            their_contribution: 13,
            amount: 24,
            proposed_by_us: true,
            is_my_turn: false,
        }],
        live_games: vec![],
        on_chain_games: vec![],
        watching_coins: vec![],
        current_height: 123,
    };

    let json = serde_json::to_value(&summary).expect("serialize");
    assert_eq!(json["state_number"], 3);
    assert_eq!(json["signed_state_number"], 2);
    assert_eq!(json["have_potato"], true);
    assert_eq!(json["current_height"], 123);
    assert!(json["channel"].is_null());
    assert_eq!(json["unroll_coin"]["state_number"], 3);
    assert_eq!(json["pending_proposals"].as_array().unwrap().len(), 1);
    assert_eq!(json["pending_proposals"][0]["game_id"], 7);
    assert_eq!(json["pending_proposals"][0]["amount"], 24);
    assert_eq!(json["pending_proposals"][0]["proposed_by_us"], true);
    assert_eq!(json["pending_proposals"][0]["is_my_turn"], false);
    assert_eq!(json["live_games"].as_array().unwrap().len(), 0);
}
