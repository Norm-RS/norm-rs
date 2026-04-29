# SMEV4 RS

[![Crates.io](https://img.shields.io/crates/v/smev4-rs.svg)](https://crates.io/crates/smev4-rs)
[![Documentation](https://docs.rs/smev4-rs/badge.svg)](https://docs.rs/smev4-rs)
[![License](https://img.shields.io/crates/l/smev4-rs.svg)](https://github.com/Norm-RS/norm-rs/blob/main/rfe/smev4-rs/LICENSE)

Asynchronous SMEV 4 (REST/OIDC) client for Russian fintech/regtech integrations.

## What this crate provides

- Queue-based SMEV 4 flow: `submit -> QueueTicket -> poll`.
- Ready-to-use service clients: FNS (`check_inn_and_income`) and ESIA (`request_user_profile`).
- Configurable polling with exponential backoff via `PollConfig`.
- Explicit state-service availability signaling via `SmevError::Unavailable`.
- Explicit ticket-expiry/not-found path (`404 -> SmevError::Payload`).
- Public unavailable classification helper: `UnavailableReason::from_http_status`.
- Audit trail helpers powered by `rfe-types::AuditEntry` and BLAKE3 request fingerprinting.
- Built-in `tracing` instrumentation in queue polling/audit flow.

## Regulatory and operational context

The crate is designed for practical SMEV 4 migration and runtime operations where teams need to:

- record verification attempts for AML/CFT (115-FZ) evidence,
- degrade predictably when state data providers are unavailable,
- control query costs with deduplication.

## Minimal usage example

```rust
use rfe_types::Inn;
use smev4_rs::{AuthProvider, PollConfig, SmevClient, SmevError};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = SmevClient::builder()
        .base_url("https://smev4-agent.example.ru/v1")
        .auth_provider(AuthProvider::Certificate {
            path: "/etc/crypto/certs/gost.pem".to_string(),
        })
        .build()?;

    let ticket = client
        .fns()
        .check_inn_and_income(Inn::new_unchecked("7700000000"), "01.01.1990")
        .await?;

    let xml = client
        .poll_response_with_config(
            ticket,
            PollConfig {
                max_attempts: 12,
                timeout_total_secs: 90,
                ..PollConfig::default()
            },
        )
        .await;

    match xml {
        Ok(body) => {
            println!("SMEV response ready: {} bytes", body.len());
        }
        Err(SmevError::Unavailable { reason }) => {
            eprintln!("State service unavailable: {reason}");
        }
        Err(e) => return Err(Box::new(e)),
    }

    Ok(())
}
```

## Chained audit example

```rust
use smev4_rs::{QueueTicket, SmevClient};

async fn audited_chain(client: &SmevClient) {
    let nonce = [1u8; 32];
    let (_r1, a1) = client
        .poll_response_chained(QueueTicket("t1".to_string()), nonce, None)
        .await;

    let (_r2, _a2) = client
        .poll_response_chained(QueueTicket("t2".to_string()), nonce, Some(&a1))
        .await;
}
```

## Current version limits

Current `smev4-rs` implements a stable SMEV4 core:

- queue-based flow and configurable polling,
- typed unavailable path with classification helper,
- audit helpers and deterministic request fingerprint.

Next-stage items planned for later release:

- unified SMEV3/SMEV4 routing API,
- structured unavailable payload directly in `SmevError`,
- stricter contract parsing across all service adapters and cache policy surfaces.

## Deduplication and audit example

```rust
use smev4_rs::{QueueTicket, SmevClient};

async fn audited_poll(client: &SmevClient, payload: &[u8], ticket: QueueTicket) {
    let fingerprint = SmevClient::request_fingerprint(payload);
    let nonce = fingerprint;

    let (_result, audit) = client.poll_response_audited(ticket, nonce).await;
    // Persist `audit` into your verification evidence chain.
    let _ = audit;
}
```

## Dependency note

`smev4-rs` depends on `rfe-types`. In the crates.io package manifest, this dependency is normalized to the published registry version (`rfe-types = "0.1.0"`) rather than a local path.
