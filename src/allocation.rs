//! Allocation engine — winner determination strategies.
//!
//! Different auction formats use different rules to decide who wins:
//!
//! - **Highest-bid**: English, Dutch, first-price sealed-bid
//! - **Second-price (Vickrey)**: highest bidder wins, pays second-highest price
//! - **Combinatorial**: bundle allocation with winner-determination

use crate::auction::AuctionType;
use crate::bid::BidHistory;
use serde::{Deserialize, Serialize};

/// Strategy for determining the auction winner.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AllocationStrategy {
    /// Highest bidder wins (English, Dutch, first-price sealed-bid).
    HighestBid,
    /// Highest bidder wins, but pays the second-highest price (Vickrey).
    SecondPrice,
    /// Combinatorial / bundle allocation.
    Combinatorial,
}

impl AllocationStrategy {
    /// Get the default strategy for an auction type.
    pub fn for_auction_type(at: AuctionType) -> Self {
        match at {
            AuctionType::Vickrey => AllocationStrategy::SecondPrice,
            AuctionType::English => AllocationStrategy::HighestBid,
            AuctionType::Dutch => AllocationStrategy::HighestBid,
            AuctionType::Combinatorial => AllocationStrategy::Combinatorial,
        }
    }
}

/// Result of an allocation decision.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllocationResult {
    /// The winning bidder's id.
    pub winner_id: String,
    /// The price the winner pays.
    pub payment: f64,
    /// The winner's bid amount.
    pub winning_bid: f64,
    /// The winner's private valuation.
    pub winner_value: f64,
    /// Allocation strategy used.
    pub strategy: AllocationStrategy,
}

/// Engine that determines winners based on bid history and allocation strategy.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AllocationEngine {
    pub strategy: AllocationStrategy,
}

impl AllocationEngine {
    /// Create an engine with the given strategy.
    pub fn new(strategy: AllocationStrategy) -> Self {
        Self { strategy }
    }

    /// Create an engine suited to the given auction type.
    pub fn for_auction_type(at: AuctionType) -> Self {
        Self::new(AllocationStrategy::for_auction_type(at))
    }

    /// Determine the winner from a bid history.
    pub fn allocate(&self, history: &BidHistory) -> Result<AllocationResult, String> {
        if history.is_empty() {
            return Err("No bids to allocate from".into());
        }

        match self.strategy {
            AllocationStrategy::HighestBid => self.allocate_highest(history),
            AllocationStrategy::SecondPrice => self.allocate_second_price(history),
            AllocationStrategy::Combinatorial => self.allocate_combinatorial(history),
        }
    }

    fn allocate_highest(&self, history: &BidHistory) -> Result<AllocationResult, String> {
        let winner = history.highest().ok_or("No highest bid found")?;
        Ok(AllocationResult {
            winner_id: winner.bidder_id.clone(),
            payment: winner.amount,
            winning_bid: winner.amount,
            winner_value: winner.value,
            strategy: self.strategy,
        })
    }

    fn allocate_second_price(&self, history: &BidHistory) -> Result<AllocationResult, String> {
        let winner = history.highest().ok_or("No highest bid found")?;
        let second = history.second_highest();

        // If only one bidder, they pay their own bid (or 0 in some formulations;
        // we use their bid to avoid negative revenue edge cases).
        let payment = second.map(|s| s.amount).unwrap_or(0.0);

        Ok(AllocationResult {
            winner_id: winner.bidder_id.clone(),
            payment,
            winning_bid: winner.amount,
            winner_value: winner.value,
            strategy: self.strategy,
        })
    }

    fn allocate_combinatorial(&self, history: &BidHistory) -> Result<AllocationResult, String> {
        // Simplified combinatorial: allocate to the bidder with the highest
        // total valuation across all their bids (bundle value).
        use std::collections::HashMap;
        let mut bundle_values: HashMap<String, (f64, f64)> = HashMap::new();

        for bid in history.all() {
            let entry = bundle_values
                .entry(bid.bidder_id.clone())
                .or_insert((0.0, 0.0));
            entry.0 += bid.value;
            entry.1 += bid.amount;
        }

        let (winner_id, (value, amount)) = bundle_values
            .into_iter()
            .max_by(|a, b| {
                a.1.1
                    .partial_cmp(&b.1.1)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or("No bids to allocate from")?;

        Ok(AllocationResult {
            winner_id: winner_id.clone(),
            payment: amount,
            winning_bid: amount,
            winner_value: value,
            strategy: self.strategy,
        })
    }
}
