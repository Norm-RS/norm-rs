# Changelog: lending-state-rs

## [0.1.0] - 2026-04-15

Initial public release. This entry consolidates all pre-release development done before first publish.

### Added

- Typestate FSM for lending lifecycle.
- Transition API: draft -> scoring -> pdn check -> approve/reject -> disburse -> close.
- BNPL controls aligned with 283-FZ threshold behavior.
- Compile-fail tests proving invalid state transitions do not compile.
- Benchmark harness for FSM execution path.
- Identification threshold check (115-FZ, > 15,000 RUB requires full identification).
- BNPL term transition support (6 months until 2028-04-01, 4 months from 2028-04-01).
- Tiered PDN risk input via `PdnRiskTier` with compatibility bool API.
