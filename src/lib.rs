//! # auction-theory
//!
//! Auction mechanism design in pure Rust: Vickrey (second-price sealed-bid),
//! English (ascending open-cry), Dutch (descending open), and combinatorial
//! allocation with full revenue-equivalence verification and incentive-compatibility
//! checks.
//!
//! ## Module overview
//!
//! | Module | Purpose |
//! |---|---|
//! | [`auction`] | Core `Auction` trait, `AuctionType` enum, auction state machine |
//! | [`bid`] | `Bid`, `BidHistory`, value/amount tracking |
//! | [`allocation`] | `AllocationEngine` — winner determination strategies |
//! | [`payment`] | `PaymentRule` — first-price, second-price, all-pay |
//! | [`revenue`] | `RevenueEquivalence` — theorem verification |
//! | [`mechanism`] | `MechanismDesign` — DSIC, efficiency, individual rationality |

pub mod allocation;
pub mod auction;
pub mod bid;
pub mod mechanism;
pub mod payment;
pub mod revenue;

pub use allocation::AllocationEngine;
pub use auction::{Auction, AuctionState, AuctionType};
pub use bid::{Bid, BidHistory};
pub use mechanism::MechanismDesign;
pub use payment::PaymentRule;
pub use revenue::RevenueEquivalence;

#[cfg(test)]
mod tests;
