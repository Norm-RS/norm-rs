# Changelog: cbr-finapi-rs

## [0.1.0] - 2026-04-15

Initial public release. This entry consolidates all pre-release development done before first publish.

### Added

- Typed AntiFraud request/decision models.
- `FraudSign` enum with 12-sign 161-FZ / OD-2506 coverage.
- Async client mode (`client` feature) with API builder.
- Retry/backoff behavior for HTTP 429 responses with `Retry-After` header support.
- Integration tests for request flow and rate-limit handling.
- Core unit tests for sign deserialization and unknown-sign forward compatibility.
- Optional anti-fraud request metadata fields for ATM, credential-change, and cross-border signals.
