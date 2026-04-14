# Changelog: rfe-types

## [0.1.0] - 2026-04-14

Initial public release. This entry consolidates all pre-release development done before first publish.

### Added

- `Sensitive<T>` wrapper for PII-safe `Debug` output.
- Newtype identifiers: `Inn`, `Ogrn`, `RequestId`, `LoanId`, `ClientId`.
- Blake3 hashing helpers and deterministic chain construction.
- Canonical audit structures: `AuditEntry`, `SealInput`, protocol-side serialization helpers.
- Audit fields used by TrustBox flow: `seal`, `processing_time_micros`, `operator_binding_hash`, `session_nonce`.
- Financial helpers via `rust_decimal` re-export.

### Notes

- `SealInput` considered frozen for v0.1.x compatibility line.
