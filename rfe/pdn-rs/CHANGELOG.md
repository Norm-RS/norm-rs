# Changelog: pdn-rs

## [0.1.0] - 2026-04-14

Initial public release. This entry consolidates all pre-release development done before first publish.

### Added

- PDN calculation core (`PdnCalculator`) aligned with 6960-U / 7226-U policy line.
- `IncomeSource` model including `Confirmed`, `Declared`, `Estimated`, `FamilyTotal`.
- Credit-card recognized payment logic using 5% limit rule fallback.
- `no_std` compatibility for embedded integration.
- Property-based and unit tests for key invariants.

### Compliance Behavior

- Unconfirmed income (`Declared` / `Estimated`) is rejected for consumer-loan PDN calculation per 6960-U / 7226-U.
- `IncomeSource::Declared` and `IncomeSource::Estimated` are treated as unconfirmed policy branches.
