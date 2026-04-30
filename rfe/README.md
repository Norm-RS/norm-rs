# RFE (Rust Fintech Ecosystem) Core

Core Rust workspace for Russian fintech compliance and lending workflows.

## Why Rust

2026 compliance workloads need deterministic and auditable execution. Rust type system and memory model reduce silent logic drift in regulation-critical paths.

## Wave Plan

### Wave 1 (April-May 2026, first publish target)

- [`rfe-types`](rfe-types/README.md)
- [`pdn-rs`](pdn-rs/README.md)
- [`cbr-finapi-rs`](cbr-finapi-rs/README.md)
- [`lending-state-rs`](lending-state-rs/README.md)

### Wave 2

- [`smev4-rs`](smev4-rs/README.md)
- `xbrl-cbr-rs` (planned)

## Layered Architecture

- Layer 0: `rfe-types`
- Layer 1: `pdn-rs`, `cbr-finapi-rs`, `smev4-rs`
- Layer 2: `lending-state-rs`
- Layer 3: `xbrl-cbr-rs`

## Development

```bash
cargo build --workspace
cargo test --workspace
```
