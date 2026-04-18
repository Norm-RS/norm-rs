use crate::SmevError;
use alloc::format;
use alloc::string::{String, ToString};
#[cfg(feature = "std")]
use reqwest::Client as HttpClient;
use rfe_types::{blake3_hash, AuditEntry};
#[cfg(feature = "std")]
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub enum AuthProvider {
    Oidc {
        token_url: String,
        client_id: String,
        client_secret: String,
    },
    Certificate {
        path: String,
    },
}

#[derive(Debug, Clone)]
pub struct QueueTicket(pub String);

#[derive(Debug, Clone, Copy)]
pub struct PollConfig {
    pub max_attempts: u32,
    pub initial_delay_ms: u64,
    pub max_delay_ms: u64,
    pub timeout_total_secs: u64,
}

impl Default for PollConfig {
    fn default() -> Self {
        Self {
            max_attempts: 10,
            initial_delay_ms: 100,
            max_delay_ms: 10_000,
            timeout_total_secs: 30,
        }
    }
}

pub struct SmevClient {
    http: HttpClient,
    base_url: String,
    _auth: AuthProvider,
}

impl SmevClient {
    pub fn builder() -> SmevClientBuilder {
        SmevClientBuilder::default()
    }

    pub async fn poll_response(&self, ticket: QueueTicket) -> Result<String, SmevError> {
        self.poll_response_with_config(ticket, PollConfig::default())
            .await
    }

    pub async fn poll_response_with_config(
        &self,
        ticket: QueueTicket,
        config: PollConfig,
    ) -> Result<String, SmevError> {
        let url = format!("{}/api/v1/queue/{}", self.base_url, ticket.0);

        let mut attempts: u32 = 0;
        let mut delay_ms = config.initial_delay_ms;
        let started = Instant::now();

        loop {
            // Simplified for demonstration: we'd attach auth headers here
            let res = self.http.get(&url).send().await?;
            let status = res.status();

            if status.is_success() {
                let text = res.text().await?;
                // Check if still processing or ready
                if !text.contains("<Status>PROCESSING</Status>") {
                    return Ok(text);
                }
            } else if status.as_u16() == 403 || status.as_u16() == 423 || status.as_u16() == 503 {
                return Err(SmevError::Unavailable {
                    reason: format!("status {}", status),
                });
            }

            attempts += 1;
            if attempts >= config.max_attempts {
                return Err(SmevError::Timeout);
            }
            if started.elapsed().as_secs() >= config.timeout_total_secs {
                return Err(SmevError::Timeout);
            }

            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            delay_ms = core::cmp::min(delay_ms.saturating_mul(2), config.max_delay_ms);
        }
    }

    pub fn request_fingerprint(payload: &[u8]) -> [u8; 32] {
        blake3_hash(payload)
    }

    pub async fn poll_response_audited(
        &self,
        ticket: QueueTicket,
        nonce: [u8; 32],
    ) -> (Result<String, SmevError>, AuditEntry) {
        let result = self.poll_response(ticket).await;
        let ts = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_micros() as u64)
            .unwrap_or(0);
        let payload = match &result {
            Ok(xml) => xml.as_bytes(),
            Err(SmevError::Unavailable { .. }) => b"SMEV_UNAVAILABLE",
            Err(SmevError::Timeout) => b"SMEV_TIMEOUT",
            Err(_) => b"SMEV_ERROR",
        };
        let audit = AuditEntry::genesis(ts, payload, Some(&nonce));
        (result, audit)
    }

    pub fn get_http(&self) -> &HttpClient {
        &self.http
    }

    pub fn get_url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub fn fns(&self) -> crate::services::FnsService<'_> {
        crate::services::FnsService::new(self)
    }

    pub fn esia(&self) -> crate::services::EsiaService<'_> {
        crate::services::EsiaService::new(self)
    }
}

#[derive(Default)]
pub struct SmevClientBuilder {
    base_url: Option<String>,
    auth: Option<AuthProvider>,
}

impl SmevClientBuilder {
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn auth_provider(mut self, auth: AuthProvider) -> Self {
        self.auth = Some(auth);
        self
    }

    pub fn build(self) -> Result<SmevClient, SmevError> {
        let base_url = self
            .base_url
            .ok_or_else(|| SmevError::Auth("base_url required for SMEV 4 client".to_string()))?;
        let auth = self.auth.unwrap_or(AuthProvider::Certificate {
            path: "/etc/crypto/certs/gost.pem".to_string(),
        });

        let http = HttpClient::builder()
            .timeout(Duration::from_secs(30))
            .build()?;

        Ok(SmevClient {
            http,
            base_url,
            _auth: auth,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_deterministic() {
        let a = SmevClient::request_fingerprint(b"payload");
        let b = SmevClient::request_fingerprint(b"payload");
        let c = SmevClient::request_fingerprint(b"payload2");
        assert_eq!(a, b);
        assert_ne!(a, c);
    }
}
