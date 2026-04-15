# lending-state-rs

Typestate FSM for lending and BNPL lifecycles with compile-time transition safety.

## State Flow

`Draft -> Scoring -> PdnCheck -> Approved/Rejected -> Disbursed -> Closed`

Only valid methods exist on each state type, so invalid transitions do not compile.

## Regulatory Controls

- 115-FZ identification threshold: amounts above 15,000 RUB require full identification.
- BNPL 283-FZ amount gate: BNPL without BKI cannot exceed 50,000 RUB.
- BNPL term gate with transition timeline:
  - max 6 months until 2028-04-01
  - max 4 months from 2028-04-01
- `PdnCheck` supports tiered risk input (`PdnRiskTier`) and keeps compatibility bool API.

## Usage

```rust
use lending_state_rs::{LoanApplication, PdnRiskTier};
use rfe_types::{rust_decimal::Decimal, ClientId};

let draft = LoanApplication::new_with_identification(
    ClientId::new(),
    Decimal::new(20_000, 0),
    true,
).unwrap();

let scoring = draft.submit_to_scoring();
let pdn = scoring.scoring_done(true).unwrap();
let approved = pdn.pdn_done_tier(PdnRiskTier::Tier15x).unwrap();
let disbursed = approved.disburse();
let _closed = disbursed.close();
```
