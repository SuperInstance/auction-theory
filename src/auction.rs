//! Auction trait, type enum, and state machine.
//!
//! Every auction shares a lifecycle: bids are collected (or rounds advance),
//! a winner is determined, and a final price is set. This module captures
//! that common interface.

use crate::bid::Bid;
use crate::payment::PaymentRule;
use serde::{Deserialize, Serialize};

/// The auction format.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AuctionType {
    /// Vickrey / second-price sealed-bid.
    Vickrey,
    /// English ascending open-cry.
    English,
    /// Dutch descending open.
    Dutch,
    /// Combinatorial (bundle) allocation.
    Combinatorial,
}

/// Snapshot of an auction at a point in time.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuctionState {
    /// Which mechanism this auction uses.
    pub auction_type: AuctionType,
    /// All bids submitted so far.
    pub bids: Vec<Bid>,
    /// Current round (0-indexed). Relevant for English/Dutch multi-round formats.
    pub round: u32,
    /// Winning bidder id, once determined.
    pub winner: Option<String>,
    /// Final price the winner pays.
    pub price: Option<f64>,
    /// Whether the auction has closed.
    pub closed: bool,
}

impl AuctionState {
    /// Create a fresh auction state for the given type.
    pub fn new(auction_type: AuctionType) -> Self {
        Self {
            auction_type,
            bids: Vec::new(),
            round: 0,
            winner: None,
            price: None,
            closed: false,
        }
    }

    /// Record a bid into this auction.
    pub fn submit_bid(&mut self, bid: Bid) {
        self.bids.push(bid);
    }

    /// Advance to the next round.
    pub fn advance_round(&mut self) {
        self.round += 1;
    }

    /// Close the auction and record results.
    pub fn close(&mut self, winner: String, price: f64) {
        self.winner = Some(winner);
        self.price = Some(price);
        self.closed = true;
    }
}

/// Core auction interface. Implementations drive the lifecycle for each format.
pub trait Auction {
    /// The auction type this implementation handles.
    fn auction_type(&self) -> AuctionType;

    /// Submit a bid. Returns `Ok(())` if the bid was accepted.
    fn submit_bid(&mut self, bid: Bid) -> Result<(), String>;

    /// Advance to the next round (English/Dutch). No-op for sealed-bid.
    fn advance_round(&mut self);

    /// Determine the current highest bid, if any.
    fn current_highest_bid(&self) -> Option<&Bid>;

    /// Close the auction and return (winner_id, price).
    fn close(&mut self) -> Result<(String, f64), String>;

    /// Get a snapshot of the current state.
    fn state(&self) -> &AuctionState;

    /// The payment rule used by this auction.
    fn payment_rule(&self) -> PaymentRule;
}
