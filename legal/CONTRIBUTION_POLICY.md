# Contribution Policy

Date: 2026-04-15
Project: NORM (`norm-rs`)

## Requirements

1. All contributors must sign [CLA](CLA.md) before non-trivial code is merged.
2. Changes must pass required CI checks.
3. Changes affecting protocol/audit semantics must include deterministic tests.
4. Regulatory text updates must use current baseline references (`6960-U/7226-U`, `161-FZ`, `283-FZ` where applicable).

## Review Rules

1. Security/compliance-impacting changes require maintainer review.
2. Breaking API/protocol changes require explicit migration notes in release docs.
3. New dependencies must include licensing and supply-chain rationale.

## Testing Baseline

Contributions should include or preserve:

- unit/integration coverage for touched logic
- deterministic behavior checks where reproducibility is expected
- frontend smoke coverage for changed UI workflows

## Prohibited Changes

- Introduction of undocumented foreign network dependencies in core regulatory paths.
- Silent fallback behavior in canonical hashing or audit verification code paths.
