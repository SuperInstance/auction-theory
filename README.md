# auction-theory

**Auction mechanism design in pure Rust.**

`auction-theory` provides the building blocks for reasoning about, simulating, and
verifying classical auction mechanisms — Vickrey (second-price sealed-bid), English
(ascending open-cry), Dutch (descending open), and combinatorial allocation — with
first-class support for the **Revenue Equivalence Theorem** and **Dominant Strategy
Incentive Compatibility (DSIC)** verification.

This is not a trading engine. It is a **theory toolkit**: every type is a clean
abstraction over the mathematical objects auction theorists work with — valuations,
bids, allocations, payments, and mechanism properties. The goal is to make auction
theory explorable, testable, and correct-by-construction.

---

## Why this crate exists

Auction theory sits at the intersection of economics, game theory, and computer
science. It underpins real-world systems worth trillions of dollars — spectrum
allocations, ad exchanges, energy markets, procurement, and increasingly, **resource
allocation among autonomous agents**.

But the math is subtle. A Vickrey auction is "just" second-price, until you need to
*prove* that truthful bidding is a dominant strategy. Revenue equivalence holds under
three precise conditions — violate any one and the theorem fails. Mechanism design
demands that you verify efficiency, individual rationality, and incentive compatibility
simultaneously.

This crate gives you:

- **Correct primitives** — `Bid`, `BidHistory`, `AllocationEngine`, `PaymentRule`,
  `RevenueEquivalence`, `MechanismDesign` — each mapping to a well-defined concept.
- **Verifiable properties** — DSIC checks with explicit deviation testing, revenue
  equivalence condition verification, efficiency and IR audits.
- **Zero magic** — no external auction solver, no black-box optimization. Every
  allocation and payment is traceable to its rule.
- **Serde everywhere** — every public type serializes, so you can log, replay, and
  analyze auction runs.

The metaphor: **agents bidding for resources**. Whether it's compute cycles on a
cluster, bandwidth in a network, or items in a marketplace, auction theory provides
the formal framework for deciding *who gets what* and *at what price*, with
mathematical guarantees about strategic behavior. This crate makes that framework
programmable.

---

## Architecture

```
                         ┌──────────────────────────────────────────┐
                         │            MechanismDesign               │
                         │  DSIC · Efficiency · Indiv. Rationality  │
                         └──────────────┬───────────────────────────┘
                                        │ audits
                         ┌──────────────▼───────────────────────────┐
                         │           RevenueEquivalence              │
                         │   Theorem verification · Revenue comp.   │
                         └──────────────┬───────────────────────────┘
                                        │ uses
                  ┌─────────────────────┼─────────────────────┐
                  │                     │                     │
       ┌──────────▼─────────┐  ┌────────▼────────┐  ┌────────▼────────┐
       │  AllocationEngine  │  │   PaymentRule    │  │    Auction      │
       │  HighestBid        │  │   FirstPrice     │  │    State        │
       │  SecondPrice       │  │   SecondPrice    │  │    Trait        │
       │  Combinatorial     │  │   AllPay         │  │    Type         │
       └──────────┬─────────┘  └────────┬────────┘  └────────┬────────┘
                  │                     │                     │
                  └─────────────────────┼─────────────────────┘
                                        │ reads
                                ┌───────▼────────┐
                                │   BidHistory    │
                                │     Bid         │
                                │  value · amount │
                                │  bidder_id · ts │
                                └────────────────┘
```

---

## Modules

| Module | Purpose | Key types |
|---|---|---|
| `auction` | Core `Auction` trait, `AuctionType` enum, `AuctionState` lifecycle | `Auction`, `AuctionType`, `AuctionState` |
| `bid` | Bid representation and history tracking | `Bid`, `BidHistory` |
| `allocation` | Winner determination strategies | `AllocationEngine`, `AllocationStrategy`, `AllocationResult` |
| `payment` | Payment rules and revenue calculation | `PaymentRule`, `PaymentResult` |
| `revenue` | Revenue Equivalence Theorem verification | `RevenueEquivalence`, `EquivalenceConditions`, `RevenueComparison` |
| `mechanism` | Mechanism design audits (DSIC, efficiency, IR) | `MechanismDesign`, `MechanismReport`, `BidderUtility` |

---

## Quick start

```rust
use auction_theory::*;

fn main() {
    // --- Vickrey (second-price sealed-bid) auction ---
    let mut history = BidHistory::new();
    history.record(Bid::truthful(100.0, "alice"));
    history.record(Bid::truthful(80.0,  "bob"));
    history.record(Bid::truthful(60.0,  "carol"));

    let engine = AllocationEngine::for_auction_type(AuctionType::Vickrey);
    let result = engine.allocate(&history).unwrap();

    assert_eq!(result.winner_id, "alice");  // highest bidder wins
    assert_eq!(result.payment, 80.0);        // pays second price

    println!("Winner: {} pays ${:.2}", result.winner_id, result.payment);
    // → Winner: alice pays $80.00
}
```

### Full mechanism audit

```rust
use auction_theory::*;
use auction_theory::allocation::AllocationStrategy;

fn main() {
    let mut history = BidHistory::new();
    history.record(Bid::truthful(100.0, "alice"));
    history.record(Bid::truthful(80.0,  "bob"));
    history.record(Bid::truthful(60.0,  "carol"));

    let mech = MechanismDesign::new(
        AllocationStrategy::SecondPrice,
        PaymentRule::SecondPrice,
    );

    let report = mech.audit(&history);

    println!("DSIC:        {}", report.is_dsic);                    // true
    println!("Efficient:   {}", report.is_efficient);               // true
    println!("IR:          {}", report.is_individually_rational);   // true

    for u in &report.bidder_utilities {
        println!("  {} → utility: {:.1} (won: {})", u.bidder_id, u.utility, u.won);
    }
    // alice → utility: 20.0 (won: true)
    // bob   → utility:  0.0 (won: false)
    // carol → utility:  0.0 (won: false)
}
```

### DSIC deviation testing

Verify that no bidder can improve their utility by deviating from truthful bidding:

```rust
use auction_theory::*;
use auction_theory::allocation::AllocationStrategy;

let mut history = BidHistory::new();
history.record(Bid::truthful(100.0, "alice"));
history.record(Bid::truthful(80.0,  "bob"));

let mech = MechanismDesign::new(
    AllocationStrategy::SecondPrice,
    PaymentRule::SecondPrice,
);

// Test a range of deviations for every bidder
let deviations = [0.0, 50.0, 80.0, 90.0, 101.0, 150.0];
let is_dsic = mech.verify_dsic_by_deviation(&history, &deviations);

assert!(is_dsic);
// No deviation from truthfulness improves any bidder's utility
```

### Revenue equivalence comparison

```rust
use auction_theory::*;

let re = RevenueEquivalence::new(4, 0.05);

// Theoretical expected revenue: (n-1)/(n+1) = 3/5 = 0.6
let theoretical = re.theoretical_revenue();
assert!((theoretical - 0.6).abs() < 1e-9);

// Compare observed revenues across formats
let comparison = re.compare_revenues(&[
    (PaymentRule::FirstPrice,  0.58),
    (PaymentRule::SecondPrice, 0.61),
]);

println!("Equivalent: {} (tolerance: {})", comparison.equivalent, comparison.tolerance);
// → Equivalent: true (tolerance: 0.05)
```

### English ascending auction

```rust
use auction_theory::*;

let mut history = BidHistory::new();
// Multi-round ascending bids
history.record(Bid::with_timestamp(100.0, 40.0, "alice", 1));
history.record(Bid::with_timestamp(90.0,  50.0, "bob",   2));
history.record(Bid::with_timestamp(100.0, 65.0, "alice", 3));
history.record(Bid::with_timestamp(90.0,  60.0, "bob",   4)); // bob drops

let engine = AllocationEngine::for_auction_type(AuctionType::English);
let result = engine.allocate(&history).unwrap();

assert_eq!(result.winner_id, "alice");
assert_eq!(result.payment, 65.0); // first-price: winner pays their last bid
```

### Dutch descending auction

```rust
use auction_theory::*;

let mut history = BidHistory::new();
// Price descends; first bidder to accept wins at that price
history.record(Bid::with_timestamp(120.0, 75.0, "alice", 1));

let engine = AllocationEngine::for_auction_type(AuctionType::Dutch);
let result = engine.allocate(&history).unwrap();

assert_eq!(result.winner_id, "alice");
assert_eq!(result.payment, 75.0); // pays the accepted price
```

### All-pay auction

```rust
use auction_theory::*;

let mut history = BidHistory::new();
history.record(Bid::with_timestamp(100.0, 60.0, "alice"));
history.record(Bid::with_timestamp(80.0,  40.0, "bob"));

let result = PaymentRule::AllPay.calculate(&history, "alice").unwrap();
assert_eq!(result.total_revenue, 100.0); // 60 + 40: everyone pays
```

### Serialization

Every public type derives `Serialize` and `Deserialize`:

```rust
use auction_theory::*;

let bid = Bid::truthful(42.0, "alice");
let json = serde_json::to_string(&bid).unwrap();
let restored: Bid = serde_json::from_str(&json).unwrap();
assert_eq!(bid, restored);
```

---

## Mathematical foundations

### The Revenue Equivalence Theorem

**Statement** (Myerson 1981, Riley & Samuelson 1981):

Any two auction mechanisms that satisfy:

1. **Efficient allocation**: the object is always awarded to the bidder with the
   highest valuation.
2. **Zero lowest-type surplus**: a bidder with the lowest possible valuation
   (type) receives zero expected surplus.

yield the **same expected revenue** for the seller, and provide each bidder type
with the **same expected surplus**.

**Formally**: Let $v_i \sim F$ be bidder $i$'s private valuation drawn i.i.d. from
distribution $F$ on $[0, 1]$. For $n$ bidders, the expected revenue is:

$$E[\text{revenue}] = E[v^{(n-1)}]$$

where $v^{(k)}$ is the $k$-th order statistic. For uniform $F$:

$$E[\text{revenue}] = \frac{n-1}{n+1}$$

This is implemented in `RevenueEquivalence::theoretical_revenue()`.

### Vickrey (Second-Price) Pricing

In a Vickrey auction, each bidder $i$ submits a sealed bid $b_i$. The highest bidder
wins and pays the second-highest bid:

$$p^* = \max_{j \neq i^*} b_j$$

where $i^* = \arg\max_i b_i$ is the winner.

**Key property**: Truthful bidding ($b_i = v_i$) is a **weakly dominant strategy**.
This is DSIC (Dominant Strategy Incentive Compatibility).

### Incentive Compatibility

A mechanism is **Dominant Strategy Incentive Compatible (DSIC)** if truthful
reporting is a (weakly) dominant strategy for every agent:

$$u_i(v_i, v_i) \geq u_i(v_i, b_i) \quad \forall b_i, \forall v_i$$

where $u_i(v_i, b_i)$ is bidder $i$'s utility when their true value is $v_i$
and they report $b_i$.

This crate verifies DSIC via explicit **deviation testing**: for each bidder and a
set of deviation amounts, it checks that no deviation improves utility over truthful
bidding (`MechanismDesign::verify_dsic_by_deviation`).

### Individual Rationality

A mechanism is **Individually Rational (IR)** if every participant's expected utility
from participation is non-negative:

$$u_i \geq 0 \quad \forall i$$

Checked by `MechanismDesign::audit()` — any negative utility is flagged as a
violation.

### Payment Rules

| Rule | Who pays | Amount | Example |
|---|---|---|---|
| First-price | Winner | Their bid | English, Dutch |
| Second-price (Vickrey) | Winner | Second-highest bid | Vickrey sealed-bid |
| All-pay | Everyone | Their bid | Lobbying, contests |

---

## Design decisions

### Why separate allocation from payment?

Allocation (who wins) and payment (what they pay) are *orthogonal* concerns in
mechanism design. The same allocation rule (highest bidder wins) can be paired with
different payment rules (first-price, second-price, all-pay). Separating them makes
it easy to mix-and-match and verify properties like revenue equivalence across
different combinations.

### Why `Bid` has both `value` and `amount`?

In auction theory, a bidder's *private valuation* ($v_i$) and their *actual bid*
($b_i$) are distinct objects. A truthful bidder sets $b_i = v_i$; a strategic bidder
may shade ($b_i < v_i$) or overbid ($b_i > v_i$). The `Bid` type captures this
distinction explicitly, enabling DSIC verification by checking whether truthful
bidding dominates all alternatives.

### Why `BidHistory` is append-only?

Auction bids are immutable events — once submitted, they cannot be modified. The
history is a log, not a mutable collection. This mirrors how real auction systems
work and simplifies reasoning about correctness.

### Why derive `Serialize` + `Deserialize` on everything?

Auction data is valuable for analysis. Serde support means you can log bids to JSON,
replay auctions from files, build APIs over auction results, and persist state
across process restarts — without any adapter code.

### Why no external dependencies beyond `serde`?

Auction theory is pure mathematics. It doesn't need linear algebra solvers, random
number generators, or network clients. Keeping the dependency tree minimal makes the
crate portable, auditable, and fast to compile. (Serde is the one exception because
serialization is a cross-cutting concern that benefits from ecosystem-standard
integration.)

---

## Combinatorial allocation

The combinatorial allocation strategy treats all bids from the same bidder as a
**bundle**: total bid amount across all items determines the winner. This is a
simplified model — full combinatorial auctions (e.g., the VCG mechanism) require
solving NP-hard winner determination problems. This crate's combinatorial mode is
suitable for pedagogical use and small-scale simulations.

```rust
use auction_theory::*;
use auction_theory::allocation::AllocationStrategy;

let mut history = BidHistory::new();
history.record(Bid::with_timestamp(40.0, 20.0, "alice", 1));
history.record(Bid::with_timestamp(30.0, 15.0, "alice", 2)); // alice total: 35
history.record(Bid::with_timestamp(50.0, 30.0, "bob",   1)); // bob total: 30

let engine = AllocationEngine::new(AllocationStrategy::Combinatorial);
let result = engine.allocate(&history).unwrap();
assert_eq!(result.winner_id, "alice"); // 35 > 30
```

---

## API stability

This crate is in `0.x` — the API may change between minor versions. The core types
(`Bid`, `BidHistory`, `AllocationEngine`, `PaymentRule`, `RevenueEquivalence`,
`MechanismDesign`) are stable in concept, but method signatures and field names may
evolve based on real-world usage feedback.

---

## License

MIT
