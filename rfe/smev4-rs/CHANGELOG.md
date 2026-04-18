# Changelog: smev4-rs

## [0.1.0] - 2026-04-18

Initial public release, including pre-release hardening and review-driven fixes.

### Added

- Async SMEV4 client foundation with queue-based workflow.
- OIDC/certificate auth provider scaffolding and service mapping.
- `PollConfig`-driven polling API via `poll_response_with_config`.
- `poll_response_audited` helper for audit trail emission using `rfe-types::AuditEntry`.
- `request_fingerprint` helper backed by `rfe_types::blake3_hash`.
- Integration tests for unavailable path and builder validation.
- XML benchmark using real `quick-xml` parse path.

### Fixed

- Removed incorrect SMEV3 default endpoint behavior; `base_url` is now required.
- Added typed `SmevError::Unavailable { reason }` mapping for 403/423/503 availability failures.
- Replaced panic-prone queue ticket extraction with shared safe parser in `services.rs`.
- Completed `RSocketFrame::encode()` handling for `Cancel` and `Error` frame variants.
- Decoupled `alloc` feature from `gost-export` in crate features.
- Removed unnecessary ticket clone in `poll_response_audited`.
- Fixed `max_attempts` off-by-one behavior (`>=` timeout guard).
- Removed dead `SmevError::Policy` variant.
- Switched elapsed tracking to `std::time::Instant`.
- Clarified `rsocket.rs` docs: simplified internal framing, not wire-compatible RSocket protocol.
- Removed redundant imports in `rsocket.rs`.
- Added tests for `FnsCheckResponse::parse_xml` behavior.
