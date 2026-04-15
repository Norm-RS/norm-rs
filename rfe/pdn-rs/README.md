# PDN RS

[![Crates.io](https://img.shields.io/crates/v/pdn-rs.svg)](https://crates.io/crates/pdn-rs)
[![Documentation](https://docs.rs/pdn-rs/badge.svg)](https://docs.rs/pdn-rs)
[![License](https://img.shields.io/crates/l/pdn-rs.svg)](https://github.com/Norm-RS/norm-rs/blob/main/rfe/pdn-rs/LICENSE)

PDN (Debt Burden Ratio) calculation crate aligned with CBR 6960-U and amendments in 7226-U.

## Regulatory Behavior

- Supports confirmed and family-total income paths.
- Enforces confirmed-income gate for consumer-loan PDN calculation regardless of loan amount.
- Applies 5% credit-card limit rule when required by obligation model.

## Quick Start

```rust
use pdn_rs::{IncomeSource, Obligation, PdnCalculator};
use rfe_types::rust_decimal::Decimal;

let obligations = vec![Obligation {
    monthly_payment: Decimal::new(15_000, 0),
    is_credit_card: false,
    credit_limit: None,
}];

let income = IncomeSource::Confirmed(Decimal::new(60_000, 0));
let new_payment = Decimal::new(10_000, 0);
let loan_amount = Decimal::new(300_000, 0);

let result = PdnCalculator::calculate(&obligations, &income, new_payment, loan_amount).unwrap();
assert_eq!(result.ratio, Decimal::new(42, 2));
```

## PDN Bands (6960-U / 7226-U)

| PDN Range | Capital Multiplier |
| --- | --- |
| 0-30% | 1.0x |
| 31-50% | 1.5x |
| 51-70% | 3.0x |
| >70% or unconfirmed income | 5.0x |
