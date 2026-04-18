use mockito::Server;
use rfe_types::Inn;
use smev4_rs::{AuthProvider, PollConfig, QueueTicket, SmevClient, SmevError};

#[tokio::test]
async fn test_smev_fns_check() {
    let mut server = Server::new_async().await;

    let mock_enqueue = server
        .mock("POST", "/api/v1/fns/check")
        .with_status(200)
        .with_header("content-type", "application/xml")
        .with_body(r#"<Response><TicketId>tkt_12345</TicketId></Response>"#)
        .create_async()
        .await;

    let mock_poll = server
        .mock("GET", "/api/v1/queue/tkt_12345")
        .with_status(200)
        .with_body(
            r#"<Response><Status>DONE</Status><IsValid>true</IsValid><IncomeConfirmed>true</IncomeConfirmed></Response>"#,
        )
        .create_async()
        .await;

    let client = SmevClient::builder()
        .base_url(server.url())
        .auth_provider(AuthProvider::Certificate {
            path: "dummy".to_string(),
        })
        .build()
        .unwrap();

    let ticket = client
        .fns()
        .check_inn_and_income(Inn::new_unchecked("7700000000"), "01.01.1990")
        .await
        .unwrap();
    assert_eq!(ticket.0, "tkt_12345");

    let result_xml = client.poll_response(ticket).await.unwrap();
    assert!(result_xml.contains("DONE"));

    mock_enqueue.assert_async().await;
    mock_poll.assert_async().await;
}

#[tokio::test]
async fn test_smev_esia_profile_request() {
    let mut server = Server::new_async().await;

    let mock_enqueue = server
        .mock("POST", "/api/v1/esia/profile")
        .with_status(200)
        .with_header("content-type", "application/xml")
        .with_body(r#"<Response><TicketId>esia_5678</TicketId></Response>"#)
        .create_async()
        .await;

    let client = SmevClient::builder()
        .base_url(server.url())
        .build()
        .unwrap();

    let ticket = client
        .esia()
        .request_user_profile("100020003000")
        .await
        .unwrap();
    assert_eq!(ticket.0, "esia_5678");

    mock_enqueue.assert_async().await;
}

#[tokio::test]
async fn test_poll_response_returns_unavailable_on_service_lock() {
    let mut server = Server::new_async().await;

    let mock_poll = server
        .mock("GET", "/api/v1/queue/locked_ticket")
        .with_status(503)
        .with_body("temporary outage")
        .create_async()
        .await;

    let client = SmevClient::builder()
        .base_url(server.url())
        .build()
        .unwrap();

    let err = client
        .poll_response_with_config(
            QueueTicket("locked_ticket".to_string()),
            PollConfig {
                max_attempts: 1,
                initial_delay_ms: 0,
                max_delay_ms: 0,
                timeout_total_secs: 1,
            },
        )
        .await
        .unwrap_err();

    match err {
        SmevError::Unavailable { reason } => assert!(reason.contains("503")),
        other => panic!("expected Unavailable, got {other:?}"),
    }

    mock_poll.assert_async().await;
}

#[test]
fn test_builder_requires_explicit_base_url() {
    let result = SmevClient::builder().build();
    let err = match result {
        Err(e) => e,
        Ok(_) => panic!("expected builder error when base_url missing"),
    };

    match err {
        SmevError::Auth(msg) => assert!(msg.contains("base_url required")),
        other => panic!("expected Auth error, got {other:?}"),
    }
}
