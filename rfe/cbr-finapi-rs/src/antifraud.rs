#[cfg(feature = "client")]
use crate::{CbrApiClient, CbrApiError};
use rfe_types::rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Serialize, Default)]
pub struct AntiFraudRequest {
    pub transaction_id: String,
    pub amount: Decimal,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sender_inn: Option<String>,
    pub receiver_account: String,

    // New fields required for 2026 fraud signatures
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_fingerprint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_bank_bic: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atm_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub atm_roundtrip_ms: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_credential_change_micros: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cross_border_transfer: Option<bool>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum FraudSign {
    ReceiverInDatabase,
    DeviceInDatabase,
    AtypicalTransaction,
    SuspiciousSbpTransfer,
    SuspiciousNfcActivity,
    MultipleAccountsFromSingleDevice,
    InconsistentGeolocation,
    HighVelocityTransfersInShortWindow,
    RemoteAccessToolDetected,
    KnownProxyOrVpnEndpoint,
    SocialEngineeringPatternDetected,
    ExternalOperatorSignal,
    #[serde(untagged)]
    Other(String),
}

#[derive(Debug, Deserialize, Default)]
pub struct AntiFraudDecision {
    pub allowed: bool,
    pub risk_score: u8,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,

    // Matched signs of fraud among the 12 criteria
    #[serde(default)]
    pub matched_fraud_signs: Vec<FraudSign>,
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "client")]
    use super::retry_after_backoff_ms;
    use super::{AntiFraudDecision, FraudSign};
    use alloc::vec;

    #[test]
    fn parses_known_fraud_sign_variant() {
        let sign: FraudSign = serde_json::from_str("\"ExternalOperatorSignal\"").unwrap();
        assert_eq!(sign, FraudSign::ExternalOperatorSignal);
    }

    #[test]
    fn parses_unknown_fraud_sign_into_other() {
        let sign: FraudSign = serde_json::from_str("\"FutureSignVariant\"").unwrap();
        assert_eq!(sign, FraudSign::Other("FutureSignVariant".into()));
    }

    #[test]
    fn decision_deserializes_sign_list() {
        let decision: AntiFraudDecision = serde_json::from_str(
            r#"{
                "allowed": false,
                "risk_score": 91,
                "matched_fraud_signs": [
                    "DeviceInDatabase",
                    "KnownProxyOrVpnEndpoint",
                    "FutureSignVariant"
                ]
            }"#,
        )
        .unwrap();

        assert!(!decision.allowed);
        assert_eq!(decision.risk_score, 91);
        assert_eq!(
            decision.matched_fraud_signs,
            vec![
                FraudSign::DeviceInDatabase,
                FraudSign::KnownProxyOrVpnEndpoint,
                FraudSign::Other("FutureSignVariant".into()),
            ]
        );
    }

    #[cfg(feature = "client")]
    #[test]
    fn retry_after_header_seconds_parsed() {
        assert_eq!(retry_after_backoff_ms(Some("2"), 1), 2_000);
        assert_eq!(retry_after_backoff_ms(Some("0"), 2), 1_000);
        assert_eq!(retry_after_backoff_ms(Some("not-a-number"), 3), 1_500);
        assert_eq!(retry_after_backoff_ms(None, 2), 1_000);
    }
}

#[cfg(feature = "client")]
pub struct AntiFraudClient<'a> {
    parent: &'a CbrApiClient,
}

#[cfg(feature = "client")]
impl<'a> AntiFraudClient<'a> {
    pub(crate) fn new(parent: &'a CbrApiClient) -> Self {
        Self { parent }
    }

    pub async fn check_transaction(
        &self,
        req: AntiFraudRequest,
    ) -> Result<AntiFraudDecision, CbrApiError> {
        let url = format!("{}/v1/antifraud/check", self.parent.base_url);

        let mut retries = 0;
        loop {
            let res = self
                .parent
                .http
                .post(&url)
                .header("Authorization", format!("Bearer {}", self.parent.api_key))
                .json(&req)
                .send()
                .await?;

            match res.status().as_u16() {
                429 => {
                    if retries >= 3 {
                        return Err(CbrApiError::RateLimited);
                    }
                    retries += 1;
                    let retry_after = res
                        .headers()
                        .get(reqwest::header::RETRY_AFTER)
                        .and_then(|v| v.to_str().ok());
                    let backoff_ms = retry_after_backoff_ms(retry_after, retries);
                    tokio::time::sleep(std::time::Duration::from_millis(backoff_ms)).await;
                    continue;
                }
                503 => return Err(CbrApiError::Unavailable),
                200..=299 => {
                    let decision: AntiFraudDecision = res.json().await?;
                    return Ok(decision);
                }
                _ => return Err(CbrApiError::Http(res.error_for_status().unwrap_err())),
            }
        }
    }
}

#[cfg(feature = "client")]
fn retry_after_backoff_ms(retry_after_header: Option<&str>, retries: u64) -> u64 {
    if let Some(raw) = retry_after_header {
        if let Ok(seconds) = raw.trim().parse::<u64>() {
            return core::cmp::max(1, seconds) * 1_000;
        }
    }
    500 * retries
}
