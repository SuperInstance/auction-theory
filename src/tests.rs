#[cfg(test)]
mod tests {
    use crate::allocation::AllocationStrategy;
    use crate::*;

    // Helper to make bids with controlled timestamps
    fn bid(value: f64, amount: f64, id: &str) -> Bid {
        Bid::with_timestamp(value, amount, id, 0)
    }

    fn truthful_bid(value: f64, id: &str) -> Bid {
        Bid::with_timestamp(value, value, id, 0)
    }

    // ─── Bid tests ───────────────────────────────────────────────

    #[test]
    fn bid_truthful_equals_value() {
        let b = truthful_bid(42.0, "alice");
        assert!(b.is_truthful());
        assert_eq!(b.shading(), 0.0);
    }

    #[test]
    fn bid_shading_positive() {
        let b = bid(100.0, 80.0, "alice");
        assert_eq!(b.shading(), 20.0);
        assert!(!b.is_truthful());
    }

    #[test]
    fn bid_shading_negative_overbidding() {
        let b = bid(50.0, 60.0, "alice");
        assert_eq!(b.shading(), -10.0);
    }

    #[test]
    fn bid_fields_correct() {
        let b = Bid::with_timestamp(30.0, 25.0, "bob", 12345);
        assert_eq!(b.value, 30.0);
        assert_eq!(b.amount, 25.0);
        assert_eq!(b.bidder_id, "bob");
        assert_eq!(b.timestamp, 12345);
    }

    // ─── BidHistory tests ────────────────────────────────────────

    #[test]
    fn history_empty() {
        let h = BidHistory::new();
        assert!(h.is_empty());
        assert_eq!(h.len(), 0);
        assert!(h.highest().is_none());
        assert!(h.second_highest().is_none());
    }

    #[test]
    fn history_record_and_len() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(10.0, "a"));
        h.record(truthful_bid(20.0, "b"));
        assert_eq!(h.len(), 2);
        assert_eq!(h.bidder_count(), 2);
    }

    #[test]
    fn history_highest_bid() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(10.0, "a"));
        h.record(truthful_bid(30.0, "b"));
        h.record(truthful_bid(20.0, "c"));
        assert_eq!(h.highest().unwrap().bidder_id, "b");
    }

    #[test]
    fn history_second_highest() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(10.0, "a"));
        h.record(truthful_bid(30.0, "b"));
        h.record(truthful_bid(20.0, "c"));
        let second = h.second_highest().unwrap();
        assert_eq!(second.bidder_id, "c");
        assert_eq!(second.amount, 20.0);
    }

    #[test]
    fn history_second_highest_single_bidder() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(50.0, "a"));
        assert!(h.second_highest().is_none());
    }

    #[test]
    fn history_by_bidder() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(10.0, "a"));
        h.record(truthful_bid(20.0, "b"));
        h.record(truthful_bid(30.0, "a"));
        let a_bids = h.by_bidder("a");
        assert_eq!(a_bids.len(), 2);
        let b_bids = h.by_bidder("b");
        assert_eq!(b_bids.len(), 1);
    }

    #[test]
    fn history_bidder_count_dedup() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(10.0, "a"));
        h.record(truthful_bid(20.0, "a"));
        h.record(truthful_bid(30.0, "b"));
        assert_eq!(h.bidder_count(), 2);
    }

    #[test]
    fn history_total_surplus() {
        let mut h = BidHistory::new();
        h.record(bid(100.0, 80.0, "a"));
        h.record(bid(50.0, 50.0, "b"));
        // surplus: 20 + 0 = 20
        assert_eq!(h.total_surplus(), 20.0);
    }

    // ─── AuctionState tests ──────────────────────────────────────

    #[test]
    fn auction_state_new_vickrey() {
        let s = AuctionState::new(AuctionType::Vickrey);
        assert_eq!(s.auction_type, AuctionType::Vickrey);
        assert!(s.bids.is_empty());
        assert_eq!(s.round, 0);
        assert!(s.winner.is_none());
        assert!(s.price.is_none());
        assert!(!s.closed);
    }

    #[test]
    fn auction_state_submit_and_advance() {
        let mut s = AuctionState::new(AuctionType::English);
        s.submit_bid(truthful_bid(10.0, "a"));
        s.advance_round();
        assert_eq!(s.bids.len(), 1);
        assert_eq!(s.round, 1);
    }

    #[test]
    fn auction_state_close() {
        let mut s = AuctionState::new(AuctionType::Dutch);
        s.close("alice".into(), 75.0);
        assert!(s.closed);
        assert_eq!(s.winner, Some("alice".into()));
        assert_eq!(s.price, Some(75.0));
    }

    // ─── AllocationEngine tests ──────────────────────────────────

    #[test]
    fn allocation_highest_bid() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(10.0, "a"));
        h.record(truthful_bid(50.0, "b"));
        h.record(truthful_bid(30.0, "c"));

        let engine = AllocationEngine::new(AllocationStrategy::HighestBid);
        let result = engine.allocate(&h).unwrap();
        assert_eq!(result.winner_id, "b");
        assert_eq!(result.payment, 50.0);
    }

    #[test]
    fn allocation_second_price() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(10.0, "a"));
        h.record(truthful_bid(50.0, "b"));
        h.record(truthful_bid(30.0, "c"));

        let engine = AllocationEngine::new(AllocationStrategy::SecondPrice);
        let result = engine.allocate(&h).unwrap();
        assert_eq!(result.winner_id, "b");
        assert_eq!(result.payment, 30.0); // second-highest
    }

    #[test]
    fn allocation_second_price_single_bidder() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(50.0, "a"));

        let engine = AllocationEngine::new(AllocationStrategy::SecondPrice);
        let result = engine.allocate(&h).unwrap();
        assert_eq!(result.winner_id, "a");
        assert_eq!(result.payment, 0.0); // no second bidder
    }

    #[test]
    fn allocation_combinatorial() {
        let mut h = BidHistory::new();
        h.record(bid(40.0, 20.0, "a")); // bundle for a
        h.record(bid(30.0, 15.0, "a")); // another for a: total 35
        h.record(bid(50.0, 30.0, "b")); // single for b

        let engine = AllocationEngine::new(AllocationStrategy::Combinatorial);
        let result = engine.allocate(&h).unwrap();
        // a bids total 35, b bids total 30 → a wins
        assert_eq!(result.winner_id, "a");
    }

    #[test]
    fn allocation_empty_history_errors() {
        let h = BidHistory::new();
        let engine = AllocationEngine::new(AllocationStrategy::HighestBid);
        assert!(engine.allocate(&h).is_err());
    }

    #[test]
    fn allocation_strategy_for_type() {
        assert_eq!(
            AllocationStrategy::for_auction_type(AuctionType::Vickrey),
            AllocationStrategy::SecondPrice
        );
        assert_eq!(
            AllocationStrategy::for_auction_type(AuctionType::English),
            AllocationStrategy::HighestBid
        );
        assert_eq!(
            AllocationStrategy::for_auction_type(AuctionType::Dutch),
            AllocationStrategy::HighestBid
        );
        assert_eq!(
            AllocationStrategy::for_auction_type(AuctionType::Combinatorial),
            AllocationStrategy::Combinatorial
        );
    }

    // ─── PaymentRule tests ───────────────────────────────────────

    #[test]
    fn payment_first_price() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(50.0, "a"));
        h.record(truthful_bid(30.0, "b"));

        let result = PaymentRule::FirstPrice.calculate(&h, "a").unwrap();
        assert_eq!(result.total_revenue, 50.0);
        assert_eq!(result.payments.len(), 1);
    }

    #[test]
    fn payment_second_price() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(50.0, "a"));
        h.record(truthful_bid(30.0, "b"));
        h.record(truthful_bid(20.0, "c"));

        let result = PaymentRule::SecondPrice.calculate(&h, "a").unwrap();
        assert_eq!(result.total_revenue, 30.0);
    }

    #[test]
    fn payment_all_pay() {
        let mut h = BidHistory::new();
        h.record(bid(50.0, 40.0, "a"));
        h.record(bid(30.0, 25.0, "b"));

        let result = PaymentRule::AllPay.calculate(&h, "a").unwrap();
        assert_eq!(result.total_revenue, 65.0); // 40 + 25
        assert_eq!(result.payments.len(), 2);
    }

    #[test]
    fn payment_empty_errors() {
        let h = BidHistory::new();
        assert!(PaymentRule::FirstPrice.calculate(&h, "a").is_err());
    }

    #[test]
    fn expected_revenue_uniform_theory() {
        // n=2: (2-1)/(2+1) = 1/3 ≈ 0.333
        let r = PaymentRule::FirstPrice.expected_revenue_uniform(2);
        assert!((r - 1.0 / 3.0).abs() < 1e-9);

        // n=3: 2/4 = 0.5
        let r = PaymentRule::SecondPrice.expected_revenue_uniform(3);
        assert!((r - 0.5).abs() < 1e-9);

        // n=5: 4/6 = 2/3
        let r = PaymentRule::AllPay.expected_revenue_uniform(5);
        assert!((r - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn expected_revenue_zero_bidders() {
        assert_eq!(PaymentRule::FirstPrice.expected_revenue_uniform(0), 0.0);
    }

    // ─── Vickrey truthfulness tests ──────────────────────────────

    #[test]
    fn vickrey_truthful_bidding_is_dsic() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(100.0, "alice"));
        h.record(truthful_bid(80.0, "bob"));
        h.record(truthful_bid(60.0, "carol"));

        let mech = MechanismDesign::new(AllocationStrategy::SecondPrice, PaymentRule::SecondPrice);
        let report = mech.audit(&h);

        assert!(report.is_dsic);
        assert!(report.is_efficient);
        assert!(report.is_individually_rational);
    }

    #[test]
    fn vickrey_winner_pays_second_price() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(100.0, "alice"));
        h.record(truthful_bid(80.0, "bob"));

        let engine = AllocationEngine::new(AllocationStrategy::SecondPrice);
        let result = engine.allocate(&h).unwrap();
        assert_eq!(result.winner_id, "alice");
        assert_eq!(result.payment, 80.0);
    }

    #[test]
    fn vickrey_winner_utility_positive() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(100.0, "alice"));
        h.record(truthful_bid(70.0, "bob"));

        let mech = MechanismDesign::new(AllocationStrategy::SecondPrice, PaymentRule::SecondPrice);
        let report = mech.audit(&h);

        let alice = report
            .bidder_utilities
            .iter()
            .find(|u| u.bidder_id == "alice")
            .unwrap();
        assert!(alice.won);
        assert!(alice.utility > 0.0); // 100 - 70 = 30
        assert!((alice.utility - 30.0).abs() < 1e-9);
    }

    #[test]
    fn vickrey_loser_utility_zero() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(100.0, "alice"));
        h.record(truthful_bid(70.0, "bob"));

        let mech = MechanismDesign::new(AllocationStrategy::SecondPrice, PaymentRule::SecondPrice);
        let report = mech.audit(&h);

        let bob = report
            .bidder_utilities
            .iter()
            .find(|u| u.bidder_id == "bob")
            .unwrap();
        assert!(!bob.won);
        assert!((bob.utility - 0.0).abs() < 1e-9);
    }

    #[test]
    fn vickrey_dsic_deviation_verification() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(100.0, "alice"));
        h.record(truthful_bid(80.0, "bob"));
        h.record(truthful_bid(60.0, "carol"));

        let mech = MechanismDesign::new(AllocationStrategy::SecondPrice, PaymentRule::SecondPrice);
        let deviations = [0.0, 50.0, 80.0, 90.0, 110.0, 150.0];
        assert!(mech.verify_dsic_by_deviation(&h, &deviations));
    }

    // ─── English ascending tests ─────────────────────────────────

    #[test]
    fn english_highest_bidder_wins() {
        let mut h = BidHistory::new();
        h.record(bid(100.0, 40.0, "a")); // round 1
        h.record(bid(100.0, 50.0, "b")); // round 2
        h.record(bid(100.0, 65.0, "a")); // round 3
        h.record(bid(100.0, 60.0, "b")); // round 4 — b drops

        let engine = AllocationEngine::new(AllocationStrategy::HighestBid);
        let result = engine.allocate(&h).unwrap();
        assert_eq!(result.winner_id, "a");
        assert_eq!(result.payment, 65.0);
    }

    #[test]
    fn english_payment_first_price() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(80.0, "a"));
        h.record(truthful_bid(90.0, "b"));

        let result = PaymentRule::FirstPrice.calculate(&h, "b").unwrap();
        assert_eq!(result.total_revenue, 90.0);
    }

    #[test]
    fn english_efficient_allocation() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(70.0, "a"));
        h.record(truthful_bid(100.0, "b"));
        h.record(truthful_bid(85.0, "c"));

        let mech = MechanismDesign::new(AllocationStrategy::HighestBid, PaymentRule::FirstPrice);
        let report = mech.audit(&h);
        assert!(report.is_efficient);
    }

    // ─── Dutch descending tests ──────────────────────────────────

    #[test]
    fn dutch_first_to_accept_wins() {
        // In a Dutch auction, the price descends. We simulate with one bid
        // (the first bidder to accept the current price).
        let mut h = BidHistory::new();
        h.record(bid(100.0, 75.0, "a")); // a accepts at 75

        let engine = AllocationEngine::new(AllocationStrategy::HighestBid);
        let result = engine.allocate(&h).unwrap();
        assert_eq!(result.winner_id, "a");
        assert_eq!(result.payment, 75.0);
    }

    #[test]
    fn dutch_winner_pays_bid_amount() {
        let mut h = BidHistory::new();
        h.record(bid(120.0, 80.0, "a"));

        let result = PaymentRule::FirstPrice.calculate(&h, "a").unwrap();
        assert_eq!(result.total_revenue, 80.0);
    }

    #[test]
    fn dutch_efficient_if_highest_value_bids() {
        let mut h = BidHistory::new();
        // Highest-value bidder accepts first (at a higher price)
        h.record(bid(120.0, 80.0, "a")); // a values at 120, accepts at 80

        let mech = MechanismDesign::new(AllocationStrategy::HighestBid, PaymentRule::FirstPrice);
        let report = mech.audit(&h);
        assert!(report.is_efficient);
    }

    #[test]
    fn dutch_not_dsic() {
        let mut h = BidHistory::new();
        h.record(bid(100.0, 80.0, "a")); // shading bid

        let mech = MechanismDesign::new(AllocationStrategy::HighestBid, PaymentRule::FirstPrice);
        let report = mech.audit(&h);
        assert!(!report.is_dsic); // first-price is not DSIC
    }

    // ─── Revenue equivalence tests ───────────────────────────────

    #[test]
    fn revenue_equivalence_theoretical_2_bidders() {
        let re = RevenueEquivalence::new(2, 0.01);
        let theoretical = re.theoretical_revenue();
        assert!((theoretical - 1.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn revenue_equivalence_theoretical_4_bidders() {
        let re = RevenueEquivalence::new(4, 0.01);
        // (4-1)/(4+1) = 3/5 = 0.6
        let theoretical = re.theoretical_revenue();
        assert!((theoretical - 0.6).abs() < 1e-9);
    }

    #[test]
    fn revenue_equivalence_compare_equal_revenues() {
        let re = RevenueEquivalence::new(3, 0.5);
        let comparison = re.compare_revenues(&[
            (PaymentRule::FirstPrice, 0.48),
            (PaymentRule::SecondPrice, 0.52),
        ]);
        assert!(comparison.equivalent); // within tolerance 0.5
    }

    #[test]
    fn revenue_equivalence_compare_different_revenues() {
        let re = RevenueEquivalence::new(3, 0.01);
        let comparison = re.compare_revenues(&[
            (PaymentRule::FirstPrice, 0.3),
            (PaymentRule::SecondPrice, 0.7),
        ]);
        assert!(!comparison.equivalent);
    }

    #[test]
    fn revenue_equivalence_conditions_hold() {
        let mut h1 = BidHistory::new();
        h1.record(truthful_bid(0.9, "a"));
        h1.record(truthful_bid(0.5, "b"));

        let mut h2 = BidHistory::new();
        h2.record(truthful_bid(0.9, "a"));
        h2.record(truthful_bid(0.5, "b"));

        let re = RevenueEquivalence::new(2, 0.1);
        let conditions = re.check_conditions(&[
            (&h1, PaymentRule::FirstPrice),
            (&h2, PaymentRule::SecondPrice),
        ]);
        assert!(conditions.symmetric_bidders);
    }

    #[test]
    fn revenue_equivalence_sample_expected() {
        // Simulate 3 truthful bidders with known values
        let mut h1 = BidHistory::new();
        h1.record(truthful_bid(0.8, "a"));
        h1.record(truthful_bid(0.6, "b"));
        h1.record(truthful_bid(0.4, "c"));

        let mut h2 = BidHistory::new();
        h2.record(truthful_bid(0.7, "a"));
        h2.record(truthful_bid(0.5, "b"));
        h2.record(truthful_bid(0.3, "c"));

        let re = RevenueEquivalence::new(3, 0.01);
        let avg_first =
            re.expected_revenue_sample(&[h1.clone(), h2.clone()], PaymentRule::FirstPrice);
        let avg_second =
            re.expected_revenue_sample(&[h1.clone(), h2.clone()], PaymentRule::SecondPrice);

        // First-price: winner pays their bid; second-price: winner pays 2nd-highest
        assert!(avg_first > avg_second);
    }

    #[test]
    fn revenue_equivalence_zero_bidders() {
        let re = RevenueEquivalence::new(0, 0.01);
        assert_eq!(re.theoretical_revenue(), 0.0);
    }

    // ─── Mechanism design tests ──────────────────────────────────

    #[test]
    fn mechanism_vickrey_full_audit() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(100.0, "a"));
        h.record(truthful_bid(80.0, "b"));
        h.record(truthful_bid(60.0, "c"));

        let mech = MechanismDesign::new(AllocationStrategy::SecondPrice, PaymentRule::SecondPrice);
        let report = mech.audit(&h);

        assert!(report.is_dsic);
        assert!(report.is_efficient);
        assert!(report.is_individually_rational);
        assert!(report.violations.is_empty());
    }

    #[test]
    fn mechanism_first_price_not_dsic() {
        let mut h = BidHistory::new();
        h.record(bid(100.0, 80.0, "a")); // shading
        h.record(bid(70.0, 50.0, "b"));

        let mech = MechanismDesign::new(AllocationStrategy::HighestBid, PaymentRule::FirstPrice);
        let report = mech.audit(&h);
        assert!(!report.is_dsic);
    }

    #[test]
    fn mechanism_all_pay_ir_violation() {
        let mut h = BidHistory::new();
        h.record(bid(50.0, 40.0, "a")); // loser pays 40
        h.record(bid(100.0, 80.0, "b")); // winner pays 80

        let mech = MechanismDesign::new(AllocationStrategy::HighestBid, PaymentRule::AllPay);
        let report = mech.audit(&h);
        // Loser (a) utility = -40 < 0 → IR violation
        assert!(!report.is_individually_rational);
    }

    #[test]
    fn mechanism_efficient_allocation() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(90.0, "a"));
        h.record(truthful_bid(100.0, "b"));
        h.record(truthful_bid(80.0, "c"));

        let mech = MechanismDesign::new(AllocationStrategy::SecondPrice, PaymentRule::SecondPrice);
        let report = mech.audit(&h);
        assert!(report.is_efficient);
        // b should win
        let winner = report.bidder_utilities.iter().find(|u| u.won).unwrap();
        assert_eq!(winner.bidder_id, "b");
    }

    #[test]
    fn mechanism_inefficient_allocation() {
        // Simulate inefficient by having lower-value bidder bid higher
        // (with HighestBid strategy, not SecondPrice)
        // Actually highest-bid with truthful bidding is efficient.
        // To get inefficient, we'd need strategic bidding:
        let mut h = BidHistory::new();
        h.record(bid(50.0, 90.0, "a")); // a values at 50 but bids 90
        h.record(bid(100.0, 80.0, "b")); // b values at 100 but bids 80

        let mech = MechanismDesign::new(AllocationStrategy::HighestBid, PaymentRule::FirstPrice);
        let report = mech.audit(&h);
        assert!(!report.is_efficient); // a wins but b values more
    }

    #[test]
    fn mechanism_bidder_utilities_correct() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(100.0, "a"));
        h.record(truthful_bid(60.0, "b"));

        let mech = MechanismDesign::new(AllocationStrategy::SecondPrice, PaymentRule::SecondPrice);
        let report = mech.audit(&h);

        let alice = report
            .bidder_utilities
            .iter()
            .find(|u| u.bidder_id == "a")
            .unwrap();
        assert!(alice.won);
        assert_eq!(alice.utility, 40.0); // 100 - 60

        let bob = report
            .bidder_utilities
            .iter()
            .find(|u| u.bidder_id == "b")
            .unwrap();
        assert!(!bob.won);
        assert_eq!(bob.utility, 0.0);
    }

    #[test]
    fn mechanism_dsic_deviation_loses() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(100.0, "a"));
        h.record(truthful_bid(80.0, "b"));

        let mech = MechanismDesign::new(AllocationStrategy::SecondPrice, PaymentRule::SecondPrice);
        // If b deviates down to 0, they still lose (good for them)
        // If b deviates up to 101, they win but pay 100 (utility = 80 - 100 = -20, worse!)
        assert!(mech.verify_dsic_by_deviation(&h, &[0.0, 50.0, 101.0, 200.0]));
    }

    // ─── Serialization round-trip tests ──────────────────────────

    #[test]
    fn serde_bid_roundtrip() {
        let b = Bid::with_timestamp(42.0, 35.0, "test", 9999);
        let json = serde_json::to_string(&b).unwrap();
        let deserialized: Bid = serde_json::from_str(&json).unwrap();
        assert_eq!(b, deserialized);
    }

    #[test]
    fn serde_auction_state_roundtrip() {
        let mut s = AuctionState::new(AuctionType::Vickrey);
        s.submit_bid(truthful_bid(50.0, "a"));
        s.close("a".into(), 30.0);

        let json = serde_json::to_string(&s).unwrap();
        let deserialized: AuctionState = serde_json::from_str(&json).unwrap();
        assert_eq!(s, deserialized);
    }

    #[test]
    fn serde_bid_history_roundtrip() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(10.0, "a"));
        h.record(truthful_bid(20.0, "b"));

        let json = serde_json::to_string(&h).unwrap();
        let deserialized: BidHistory = serde_json::from_str(&json).unwrap();
        assert_eq!(h, deserialized);
    }

    // ─── Edge case tests ─────────────────────────────────────────

    #[test]
    fn equal_bids_highest_wins() {
        let mut h = BidHistory::new();
        h.record(truthful_bid(50.0, "a"));
        h.record(truthful_bid(50.0, "b"));

        let engine = AllocationEngine::new(AllocationStrategy::HighestBid);
        let result = engine.allocate(&h).unwrap();
        // Either a or b should win
        assert!(result.winner_id == "a" || result.winner_id == "b");
    }

    #[test]
    fn negative_bid_amounts() {
        let mut h = BidHistory::new();
        h.record(bid(10.0, -5.0, "a"));
        h.record(bid(20.0, 5.0, "b"));

        let engine = AllocationEngine::new(AllocationStrategy::HighestBid);
        let result = engine.allocate(&h).unwrap();
        assert_eq!(result.winner_id, "b");
    }

    #[test]
    fn many_bidders_allocation() {
        let mut h = BidHistory::new();
        for i in 0..100 {
            h.record(truthful_bid(i as f64, &format!("bidder_{}", i)));
        }

        let engine = AllocationEngine::new(AllocationStrategy::SecondPrice);
        let result = engine.allocate(&h).unwrap();
        assert_eq!(result.winner_id, "bidder_99");
        // Second-highest is bidder_98 with value 98.0
        assert!((result.payment - 98.0).abs() < 1e-9);
    }

    #[test]
    fn revenue_comparison_single_format() {
        let re = RevenueEquivalence::new(3, 0.01);
        let comp = re.compare_revenues(&[(PaymentRule::FirstPrice, 0.5)]);
        assert!(comp.equivalent); // single format is trivially equivalent
    }

    #[test]
    fn auction_types_distinct() {
        assert_ne!(AuctionType::Vickrey, AuctionType::English);
        assert_ne!(AuctionType::English, AuctionType::Dutch);
        assert_ne!(AuctionType::Dutch, AuctionType::Combinatorial);
    }

    #[test]
    fn payment_rules_distinct() {
        assert_ne!(PaymentRule::FirstPrice, PaymentRule::SecondPrice);
        assert_ne!(PaymentRule::SecondPrice, PaymentRule::AllPay);
        assert_ne!(PaymentRule::FirstPrice, PaymentRule::AllPay);
    }
}
