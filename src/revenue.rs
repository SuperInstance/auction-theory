//! Revenue equivalence theorem verification.
//!
//! The **Revenue Equivalence Theorem** (Myerson, 1981; Riley & Samuelson, 1981)
//! states that any auction mechanism satisfying:
//!
//! 1. The highest-valued bidder wins.
//! 2. A bidder with the lowest possible valuation gets zero expected surplus.
//!
//! yields the **same expected revenue** for the seller.
//!
//! This module provides tools to verify that property numerically across
//! simulated auction runs.

use crate::bid::BidHistory;
use crate::payment::PaymentRule;
use serde::{Deserialize, Serialize};

/// Conditions required for revenue equivalence to hold.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EquivalenceConditions {
    /// Do bidders with the same valuations participate in all formats?
    pub symmetric_bidders: bool,
    /// Is the item always allocated to the highest-valuation bidder?
    pub efficient_allocation: bool,
    /// Does the lowest-type bidder earn zero expected surplus?
    pub zero_lowest_surplus: bool,
}

impl EquivalenceConditions {
    /// Check whether all conditions for revenue equivalence are met.
    pub fn holds(&self) -> bool {
        self.symmetric_bidders && self.efficient_allocation && self.zero_lowest_surplus
    }
}

/// Result of comparing revenues across multiple auction formats.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevenueComparison {
    /// (payment rule, observed revenue) pairs.
    pub revenues: Vec<(PaymentRule, f64)>,
    /// Theoretical expected revenue under uniform [0,1] valuations.
    pub theoretical_revenue: f64,
    /// Whether all observed revenues are within `tolerance` of each other.
    pub equivalent: bool,
    /// Tolerance used for equivalence check.
    pub tolerance: f64,
}

/// Revenue equivalence verifier.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RevenueEquivalence {
    /// Number of bidders.
    pub n_bidders: usize,
    /// Tolerance for floating-point comparison.
    pub tolerance: f64,
}

impl RevenueEquivalence {
    /// Create a new verifier for `n_bidders` with the given tolerance.
    pub fn new(n_bidders: usize, tolerance: f64) -> Self {
        Self {
            n_bidders,
            tolerance,
        }
    }

    /// Theoretical expected revenue under the revenue equivalence theorem
    /// for uniform [0,1] valuations: (n-1)/(n+1).
    pub fn theoretical_revenue(&self) -> f64 {
        if self.n_bidders == 0 {
            return 0.0;
        }
        (self.n_bidders - 1) as f64 / (self.n_bidders + 1) as f64
    }

    /// Check the three conditions for revenue equivalence.
    pub fn check_conditions(
        &self,
        histories: &[(&BidHistory, PaymentRule)],
    ) -> EquivalenceConditions {
        // Symmetric: same number of bidders in each format
        let bidder_counts: Vec<usize> = histories.iter().map(|(h, _)| h.bidder_count()).collect();
        let symmetric = bidder_counts.windows(2).all(|w| w[0] == w[1]);

        // Efficient: highest-valuation bidder always wins (check each history)
        let efficient = histories.iter().all(|(h, rule)| {
            if h.is_empty() {
                return true;
            }
            let highest_bid = h.highest().unwrap();
            let highest_valuer = h
                .all()
                .iter()
                .max_by(|a, b| {
                    a.value
                        .partial_cmp(&b.value)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();

            // For standard auctions the highest bidder wins, which should be
            // the highest valuer when bidders are truthful.
            let winner_valuation = highest_bid.value;
            let top_valuation = highest_valuer.value;

            // Allow small tolerance for floating point
            (winner_valuation - top_valuation).abs() < self.tolerance ||
            // In second-price, the winner is still the highest bidder
            *rule == PaymentRule::SecondPrice && highest_bid.bidder_id == highest_valuer.bidder_id
        });

        // Lowest-type surplus: the lowest-valued bidder should have ~0 surplus
        // (Approximation — checked by seeing if the min-value bidder's surplus is
        // negligible relative to the scale)
        let zero_surplus = histories.iter().all(|(h, _)| {
            if h.is_empty() {
                return true;
            }
            let min_bidder = h
                .all()
                .iter()
                .min_by(|a, b| {
                    a.value
                        .partial_cmp(&b.value)
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .unwrap();
            min_bidder.value < self.tolerance
                || min_bidder.value < min_bidder.amount + self.tolerance
        });

        EquivalenceConditions {
            symmetric_bidders: symmetric,
            efficient_allocation: efficient,
            zero_lowest_surplus: zero_surplus,
        }
    }

    /// Compare revenues across multiple auction runs and check equivalence.
    pub fn compare_revenues(&self, results: &[(PaymentRule, f64)]) -> RevenueComparison {
        let theoretical = self.theoretical_revenue();

        let all_close = if results.len() < 2 {
            true
        } else {
            let first = results[0].1;
            results
                .iter()
                .all(|(_, r)| (r - first).abs() < self.tolerance)
        };

        RevenueComparison {
            revenues: results.to_vec(),
            theoretical_revenue: theoretical,
            equivalent: all_close,
            tolerance: self.tolerance,
        }
    }

    /// Simulate expected revenue from a sample of bid histories.
    /// Computes average revenue across histories for a given payment rule.
    pub fn expected_revenue_sample(&self, histories: &[BidHistory], rule: PaymentRule) -> f64 {
        if histories.is_empty() {
            return 0.0;
        }

        let total: f64 = histories
            .iter()
            .map(|h| {
                if h.is_empty() {
                    return 0.0;
                }
                match rule {
                    PaymentRule::FirstPrice => h.highest().map(|b| b.amount).unwrap_or(0.0),
                    PaymentRule::SecondPrice => h
                        .second_highest()
                        .map(|b| b.amount)
                        .unwrap_or_else(|| h.highest().map(|b| b.amount).unwrap_or(0.0)),
                    PaymentRule::AllPay => h.all().iter().map(|b| b.amount).sum(),
                }
            })
            .sum();

        total / histories.len() as f64
    }
}
