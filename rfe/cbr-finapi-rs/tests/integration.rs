use cbr_finapi_rs::{antifraud::AntiFraudRequest, CbrApiClient, CbrApiError};
use mockito::Server;
use rfe_types::rust_decimal::Decimal;
use std::net::TcpListener;

fn loopback_bind_available() -> bool {
    TcpListener::bind("127.0.0.1:0").is_ok()
}

#[tokio::test]
async fn test_antifraud_check() {
    if !loopback_bind_available() {
        eprintln!("skipping: loopback bind unavailable in current environment");
        return;
    }

    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/antifraud/check")
        .with_status(200)
        .with_header("content-type", "application/json")
        .with_body(r#"{"allowed":true,"risk_score":10}"#)
        .create_async()
        .await;

    let client = CbrApiClient::builder()
        .base_url(server.url())
        .api_key("test_key")
        .build()
        .unwrap();

    let req = AntiFraudRequest {
        transaction_id: "tx-123".into(),
        amount: Decimal::new(15000, 0),
        sender_inn: None,
        receiver_account: "40817810000000000000".into(),
        device_fingerprint: Some("dev-abc-123".into()),
        receiver_bank_bic: Some("044525225".into()),
        atm_id: None,
        atm_roundtrip_ms: None,
        last_credential_change_micros: None,
        cross_border_transfer: None,
    };

    let decision = client.antifraud().check_transaction(req).await.unwrap();

    assert!(decision.allowed);
    assert_eq!(decision.risk_score, 10);
    assert_eq!(decision.reason, None);

    mock.assert_async().await;
}

#[tokio::test]
async fn test_rate_limiting() {
    if !loopback_bind_available() {
        eprintln!("skipping: loopback bind unavailable in current environment");
        return;
    }

    let mut server = Server::new_async().await;

    let mock = server
        .mock("POST", "/v1/antifraud/check")
        .with_status(429)
        .expect(4)
        .create_async()
        .await;

    let client = CbrApiClient::builder()
        .base_url(server.url())
        .api_key("test_key")
        .build()
        .unwrap();

    let req = AntiFraudRequest {
        transaction_id: "tx-124".into(),
        amount: Decimal::new(1000, 0),
        sender_inn: None,
        receiver_account: "40817810000000000000".into(),
        device_fingerprint: None,
        receiver_bank_bic: None,
        atm_id: None,
        atm_roundtrip_ms: None,
        last_credential_change_micros: None,
        cross_border_transfer: None,
    };

    let result = client.antifraud().check_transaction(req).await;

    assert!(matches!(result, Err(CbrApiError::RateLimited)));

    mock.assert_async().await;
}
