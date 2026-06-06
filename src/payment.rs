//! Payment rules — how much the winner (and sometimes losers) pay.
//!
//! The three canonical rules:
//!
//! - **First-price**: winner pays their bid.
//! - **Second-price (Vickrey)**: winner pays the second-highest bid.
//! - **All-pay**: every bidder pays their bid, winner gets the item.

use crate::bid::BidHistory;
use serde::{Deserialize, Serialize};

/// How payments are determined.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PaymentRule {
    /// Winner pays exactly what they bid.
    FirstPrice,
    /// Winner pays the second-highest bid amount (Vickrey).
    SecondPrice,
    /// Every bidder pays their bid regardless of outcome.
    AllPay,
}

/// Result of payment calculation for a single auction.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PaymentResult {
    /// The rule applied.
    pub rule: PaymentRule,
    /// (bidder_id, amount owed) for every participant who owes payment.
    pub payments: Vec<(String, f64)>,
    /// Total revenue collected.
    pub total_revenue: f64,
}

impl PaymentRule {
    /// Calculate payments for an auction, given the bid history and the
    /// winning bidder id.
    pub fn calculate(
        &self,
        history: &BidHistory,
        winner_id: &str,
    ) -> Result<PaymentResult, String> {
        if history.is_empty() {
            return Err("No bids to calculate payments from".into());
        }

        match self {
            PaymentRule::FirstPrice => self.calc_first_price(history, winner_id),
            PaymentRule::SecondPrice => self.calc_second_price(history, winner_id),
            PaymentRule::AllPay => self.calc_all_pay(history),
        }
    }

    fn calc_first_price(
        &self,
        history: &BidHistory,
        winner_id: &str,
    ) -> Result<PaymentResult, String> {
        let winner_bid = history
            .by_bidder(winner_id)
            .into_iter()
            .max_by(|a, b| {
                a.amount
                    .partial_cmp(&b.amount)
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .ok_or("Winner has no bids")?;

        let payment = winner_bid.amount;
        Ok(PaymentResult {
            rule: *self,
            payments: vec![(winner_id.to_string(), payment)],
            total_revenue: payment,
        })
    }

    fn calc_second_price(
        &self,
        history: &BidHistory,
        winner_id: &str,
    ) -> Result<PaymentResult, String> {
        let second = history.second_highest();
        let payment = second.map(|b| b.amount).unwrap_or(0.0);

        Ok(PaymentResult {
            rule: *self,
            payments: vec![(winner_id.to_string(), payment)],
            total_revenue: payment,
        })
    }

    fn calc_all_pay(&self, history: &BidHistory) -> Result<PaymentResult, String> {
        use std::collections::HashMap;
        let mut per_bidder: HashMap<String, f64> = HashMap::new();
        for bid in history.all() {
            *per_bidder.entry(bid.bidder_id.clone()).or_insert(0.0) += bid.amount;
        }

        let payments: Vec<(String, f64)> = per_bidder.into_iter().collect();
        let total: f64 = payments.iter().map(|(_, p)| p).sum();

        Ok(PaymentResult {
            rule: *self,
            payments,
            total_revenue: total,
        })
    }

    /// Compute expected revenue under the revenue equivalence theorem for
    /// n bidders with valuations uniformly distributed on [0, 1].
    /// All standard auction formats yield E[revenue] = (n-1) / (n+1).
    pub fn expected_revenue_uniform(&self, n_bidders: usize) -> f64 {
        if n_bidders == 0 {
            return 0.0;
        }
        // Revenue equivalence: E[second-highest order statistic] = (n-1)/(n+1)
        (n_bidders - 1) as f64 / (n_bidders + 1) as f64
    }
}
