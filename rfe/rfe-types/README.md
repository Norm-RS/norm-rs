# rfe-types

[![Crates.io](https://img.shields.io/crates/v/rfe-types.svg)](https://crates.io/crates/rfe-types)
[![Documentation](https://docs.rs/rfe-types/badge.svg)](https://docs.rs/rfe-types)
[![License](https://img.shields.io/crates/l/rfe-types.svg)](https://github.com/Norm-RS/norm-rs/blob/main/rfe/rfe-types/LICENSE)

Core domain types and deterministic audit primitives for the Rust Fintech Ecosystem (RFE).

## Key Capabilities

- `Sensitive<T>` redacts PII in `Debug` output.
- Typed IDs: `Inn`, `Ogrn`, `RequestId`, `LoanId`, `ClientId`.
- Deterministic Blake3 helpers: `blake3_hash`, `blake3_chain`.
- Canonical audit structures: `AuditEntry`, `SealInput`.
- `rust_decimal` re-export for exact financial arithmetic.

## Freeze Notice (v0.1.x)

`SealInput` is frozen starting from `v0.1.0` for TrustBox compatibility.

Field-level breaking changes require semver-major bump.

## SealInput and AuditEntry

`SealInput` fields:

- `nonce`
- `request_hash`
- `result_hash`
- `chain_head_pre`

`AuditEntry` includes:

- `seal`
- `processing_time_micros`
- `operator_binding_hash`
- `session_nonce`

## Minimal Example

```rust
use rfe_types::{blake3_hash, AuditEntry, SealInput};

let payload = b"tx:123";
let request_hash = blake3_hash(payload);
let result_hash = blake3_hash(b"allow");
let parent = [0u8; 32];
let nonce = [7u8; 32];

let seal = SealInput::new_v1(nonce, request_hash, result_hash, parent).compute_seal();
assert_ne!(seal, [0u8; 32]);

let root = AuditEntry::genesis(1_700_000_000_000_000, payload, Some(&nonce));
let next = root.next(1_700_000_000_001_000, b"tx:124", Some(&nonce));
assert!(next.verify_chain(&root));
```
