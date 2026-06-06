//! Bid representation and history tracking.
//!
//! A [`Bid`] captures what a bidder offered, when, and their true private
//! valuation. The `value` field is the bidder's private information (what
//! they think the item is worth), while `amount` is what they actually bid.

use serde::{Deserialize, Serialize};
use std::time::SystemTime;

/// A single bid in an auction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Bid {
    /// The bidder's private valuation (true willingness to pay).
    pub value: f64,
    /// The amount actually bid. May differ from `value` depending on strategy.
    pub amount: f64,
    /// Unique identifier for the bidder.
    pub bidder_id: String,
    /// When the bid was placed (seconds since Unix epoch).
    pub timestamp: u64,
}

impl Bid {
    /// Create a new bid. Uses the current system time for the timestamp.
    pub fn new(value: f64, amount: f64, bidder_id: impl Into<String>) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        Self {
            value,
            amount,
            bidder_id: bidder_id.into(),
            timestamp,
        }
    }

    /// Create a truthful bid where `amount == value`.
    pub fn truthful(value: f64, bidder_id: impl Into<String>) -> Self {
        Self::new(value, value, bidder_id)
    }

    /// Create a bid with an explicit timestamp (useful in tests).
    pub fn with_timestamp(
        value: f64,
        amount: f64,
        bidder_id: impl Into<String>,
        timestamp: u64,
    ) -> Self {
        Self {
            value,
            amount,
            bidder_id: bidder_id.into(),
            timestamp,
        }
    }

    /// How much the bidder is shading their bid (value - amount).
    /// Positive means they bid less than their value.
    pub fn shading(&self) -> f64 {
        self.value - self.amount
    }

    /// Whether this is a truthful bid (amount == value).
    pub fn is_truthful(&self) -> bool {
        (self.value - self.amount).abs() < f64::EPSILON
    }
}

/// Ordered history of bids in an auction.
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct BidHistory {
    bids: Vec<Bid>,
}

impl BidHistory {
    /// Create an empty history.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a bid.
    pub fn record(&mut self, bid: Bid) {
        self.bids.push(bid);
    }

    /// All bids, in order of submission.
    pub fn all(&self) -> &[Bid] {
        &self.bids
    }

    /// Bids submitted by a specific bidder.
    pub fn by_bidder(&self, bidder_id: &str) -> Vec<&Bid> {
        self.bids
            .iter()
            .filter(|b| b.bidder_id == bidder_id)
            .collect()
    }

    /// The highest bid by amount.
    pub fn highest(&self) -> Option<&Bid> {
        self.bids.iter().max_by(|a, b| {
            a.amount
                .partial_cmp(&b.amount)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
    }

    /// The second-highest bid by amount.
    pub fn second_highest(&self) -> Option<&Bid> {
        if self.bids.len() < 2 {
            return None;
        }
        // Find the two largest by amount
        let mut sorted: Vec<&Bid> = self.bids.iter().collect();
        sorted.sort_by(|a, b| {
            b.amount
                .partial_cmp(&a.amount)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        // Return the second element, but skip if same bidder as first
        for bid in sorted.iter().skip(1) {
            if bid.bidder_id != sorted[0].bidder_id {
                return Some(*bid);
            }
        }
        // If all from same bidder, just return second sorted
        sorted.into_iter().nth(1)
    }

    /// Number of bids recorded.
    pub fn len(&self) -> usize {
        self.bids.len()
    }

    /// Whether any bids have been recorded.
    pub fn is_empty(&self) -> bool {
        self.bids.is_empty()
    }

    /// Number of unique bidders.
    pub fn bidder_count(&self) -> usize {
        let mut ids: Vec<&str> = self.bids.iter().map(|b| b.bidder_id.as_str()).collect();
        ids.sort();
        ids.dedup();
        ids.len()
    }

    /// Total surplus across all bids (sum of value - amount for truthful bids).
    pub fn total_surplus(&self) -> f64 {
        self.bids.iter().map(|b| b.value - b.amount).sum()
    }
}
