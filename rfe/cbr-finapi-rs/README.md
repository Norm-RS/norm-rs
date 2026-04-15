# CBR FinAPI RS

[![Crates.io](https://img.shields.io/crates/v/cbr-finapi-rs.svg)](https://crates.io/crates/cbr-finapi-rs)
[![Documentation](https://docs.rs/cbr-finapi-rs/badge.svg)](https://docs.rs/cbr-finapi-rs)
[![License](https://img.shields.io/crates/l/cbr-finapi-rs.svg)](https://github.com/Norm-RS/norm-rs/blob/main/rfe/cbr-finapi-rs/LICENSE)

Typed Rust client for CBR public interfaces (AntiFraud, TSPI, EBS).

## Regulatory Scope

Implements AntiFraud integration under 161-FZ with 12-sign OD-2506 model.

## Features

- Typed request and decision models.
- Async client mode via `client` feature.
- Built-in `429` handling with `Retry-After` support (fallback linear backoff).
- Forward-compatible unknown fraud sign parsing.
- Optional request metadata for ATM/credential/cross-border signals.

## OD-2506 Fraud Signs (12/12)

| OD-2506 Sign | Enum Variant | Meaning |
| --- | --- | --- |
| 1 | `ReceiverInDatabase` | Receiver account linked to known fraud indicators. |
| 2 | `DeviceInDatabase` | Device linked to known fraud indicators. |
| 3 | `AtypicalTransaction` | Deviation from normal customer transaction profile. |
| 4 | `SuspiciousSbpTransfer` | Suspicious fast-payment pattern. |
| 5 | `SuspiciousNfcActivity` | Suspicious NFC/token usage pattern. |
| 6 | `MultipleAccountsFromSingleDevice` | One device tied to many unrelated accounts. |
| 7 | `InconsistentGeolocation` | Geo/location data inconsistent with expected behavior. |
| 8 | `HighVelocityTransfersInShortWindow` | Unusual transfer burst in short period. |
| 9 | `RemoteAccessToolDetected` | Indicators of remote-control session malware. |
| 10 | `KnownProxyOrVpnEndpoint` | Connection endpoint flagged as anonymization/proxy risk. |
| 11 | `SocialEngineeringPatternDetected` | Signals of social-engineering coercion pattern. |
| 12 | `ExternalOperatorSignal` | Signal from another payment system operator. |

## Usage

```rust,no_run
use cbr_finapi_rs::antifraud::AntiFraudRequest;
use cbr_finapi_rs::CbrApiClient;
use rfe_types::rust_decimal::Decimal;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = CbrApiClient::builder().api_key("...").build()?;

    let req = AntiFraudRequest {
        transaction_id: "tx-001".into(),
        amount: Decimal::new(10_000, 0),
        sender_inn: None,
        receiver_account: "40817810000000000000".into(),
        device_fingerprint: None,
        receiver_bank_bic: None,
        atm_id: None,
        atm_roundtrip_ms: None,
        last_credential_change_micros: None,
        cross_border_transfer: None,
    };

    let _decision = client.antifraud().check_transaction(req).await?;
    Ok(())
}
```
