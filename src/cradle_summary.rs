//! Read-only summary types for the cradle, returned in one shot by
//! [`SynchronousGameCradle::get_summary`]. Designed for cheap UI reads —
//! every field comes from already-cached in-memory state, no
//! serialization round-trip, no allocator required.
//!
//! See `docs/superpowers/specs/2026-04-20-cradle-summary-endpoint-design.md`
//! for the rationale behind the encoding choices (in particular why
//! `channel.coin` is bytes while other `*_string` / `*_hash` fields
//! are hex).

use hex::encode as hex_encode;

use serde::{Deserialize, Serialize};

use crate::channel_handler::types::{LiveGame, OnChainGameState, ProposedGame, UnrollCoin};
use crate::channel_handler::ChannelHandler;
use crate::common::types::CoinString;
use crate::peer_container::WatchEntry;
use crate::potato_handler::effects::ChannelStatusSnapshot;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CradleSummary {
    pub channel: Option<ChannelStatusSnapshot>,
    pub state_number: Option<usize>,
    pub signed_state_number: Option<usize>,
    pub have_potato: bool,
    pub unroll_coin: Option<UnrollCoinSummary>,
    pub pending_proposals: Vec<PendingProposalSummary>,
    pub live_games: Vec<LiveGameSummary>,
    pub on_chain_games: Vec<OnChainGameSummary>,
    pub watching_coins: Vec<WatchingCoinSummary>,
    pub current_height: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnrollCoinSummary {
    pub state_number: usize,
    pub conditions_hash: Option<String>,
}

impl UnrollCoinSummary {
    pub fn from_unroll(unroll: &UnrollCoin) -> Self {
        UnrollCoinSummary {
            state_number: unroll.state_number,
            conditions_hash: unroll.outcome.as_ref().map(|o| hex_encode(o.hash.bytes())),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PendingProposalSummary {
    pub game_id: u64,
    pub my_contribution: u64,
    pub their_contribution: u64,
    pub amount: u64,
    pub proposed_by_us: bool,
    pub is_my_turn: bool,
}

impl PendingProposalSummary {
    pub fn from_proposed_game(ch: &ChannelHandler, p: &ProposedGame) -> Self {
        PendingProposalSummary {
            game_id: p.game_id.0,
            my_contribution: p.my_contribution.to_u64(),
            their_contribution: p.their_contribution.to_u64(),
            amount: (p.my_contribution.clone() + p.their_contribution.clone()).to_u64(),
            proposed_by_us: ch.is_our_nonce_parity(&p.game_id),
            is_my_turn: p.referee.is_my_turn(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiveGameSummary {
    pub game_id: u64,
    pub last_referee_puzzle_hash: String,
    pub my_contribution: u64,
    pub their_contribution: u64,
    pub our_current_share: Option<u64>,
    pub their_current_share: Option<u64>,
    pub max_move_size: u64,
    pub is_my_turn: bool,
    pub is_game_over: bool,
    pub game_timeout: u64,
}

impl LiveGameSummary {
    pub fn from_live_game(g: &LiveGame) -> Self {
        let our_share = g.get_our_current_share().ok().map(|a| a.to_u64());
        let total = g.get_amount().to_u64();
        let their_share = our_share.map(|o| total.saturating_sub(o));
        LiveGameSummary {
            game_id: g.game_id.0,
            last_referee_puzzle_hash: hex_encode(g.last_referee_puzzle_hash.bytes()),
            my_contribution: g.my_contribution.to_u64(),
            their_contribution: g.their_contribution.to_u64(),
            our_current_share: our_share,
            their_current_share: their_share,
            max_move_size: g.get_max_move_size() as u64,
            is_my_turn: g.is_my_turn(),
            is_game_over: g.is_game_over(),
            game_timeout: g.get_game_timeout().to_u64(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnChainGameSummary {
    pub game_id: u64,
    pub coin_string: String,
    pub puzzle_hash: String,
    pub state_number: usize,
    pub our_turn: bool,
    pub game_timeout: u64,
    pub accepted: bool,
    pub pending_slash_amount: Option<u64>,
}

impl OnChainGameSummary {
    pub fn from_parts(coin: &CoinString, state: &OnChainGameState) -> Self {
        OnChainGameSummary {
            game_id: state.game_id.0,
            coin_string: hex_encode(coin.to_bytes()),
            puzzle_hash: hex_encode(state.puzzle_hash.bytes()),
            state_number: state.state_number,
            our_turn: state.our_turn,
            game_timeout: state.game_timeout.to_u64(),
            accepted: state.accepted,
            pending_slash_amount: state.pending_slash_amount.as_ref().map(|a| a.to_u64()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WatchingCoinSummary {
    pub coin_id: String,
    pub coin_string: String,
    pub timeout_blocks: u64,
    pub name: Option<String>,
}

impl WatchingCoinSummary {
    pub fn from_parts(coin: &CoinString, entry: &WatchEntry) -> Self {
        WatchingCoinSummary {
            coin_id: hex_encode(coin.to_coin_id().bytes()),
            coin_string: hex_encode(coin.to_bytes()),
            timeout_blocks: entry.timeout_blocks.to_u64(),
            name: entry.name.clone(),
        }
    }
}
