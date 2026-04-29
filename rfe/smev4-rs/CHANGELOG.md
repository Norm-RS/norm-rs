# Changelog: smev4-rs

## [0.1.1] - 2026-04-28

### Added

- New public `UnavailableReason` enum and classifier helper `UnavailableReason::from_http_status`.
- New chained audit helper `poll_response_chained(ticket, nonce, previous)` for continuous `AuditEntry` linking.
- New strict parser method `FnsCheckResponse::parse_xml_strict`.
- New benchmark `fns_parse_xml_strict`.
- New tests for:
- unavailable reason mapping,
- strict XML parsing success/failure,
- XML escaping behavior,
- `429 -> Unavailable` mapping,
- chained audit continuity.

### Changed

- `poll_response_audited` is now implemented via `poll_response_chained(..., None)` for consistent behavior.
- Unavailable mapping now also covers HTTP `429` and includes reason classification in the error message.
- Added `tracing` instrumentation in queue polling and chained-audit flow.

### Fixed

- XML payload construction now escapes values before insertion in FNS/ESIA request builders to prevent malformed XML and injection-prone payloads.
- Polling now returns explicit `Payload` error for HTTP `404` queue ticket path (not found/expired) instead of timing out after retries.
- Removed redundant `core::convert::Into` import in `rsocket.rs`.

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
