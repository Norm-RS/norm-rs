# NORM Monorepo: The RegTech Power Trinity

Welcome to the **NORM Ecosystem** monorepo `norm-rs`.

This monorepo unites the domain logic, analyst tooling, and hardware-hardened execution environment required to provide unfalsifiable regulatory compliance for the Russian Fintech market (2026 standards).

## Components

1. **RFE ([`rfe/`](rfe/README.md))**
   The core domain logic framework implementing Central Bank of Russia directives.
   - [`rfe-types`](rfe/rfe-types/README.md): Core domain types and deterministic audit primitives for the Rust Fintech Ecosystem (RFE).
   - [`pdn-rs`](rfe/pdn-rs/README.md): Implements 6960-U / 7226-U (Debt Service-to-Income / PDN calculations).
