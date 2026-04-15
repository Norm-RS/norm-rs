//! Typed client for CBR public APIs.
//!
//! Includes:
//! - GIS AntiFraud (mandatory from 2026-03-01)
//! - TsPI (Citizen Digital Profile)
//! - EBS (Unified Biometric System)

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;
#[cfg(feature = "client")]
use alloc::string::String;

pub mod antifraud;
#[cfg(feature = "client")]
pub mod ebs;
#[cfg(feature = "client")]
pub mod tspi;

#[cfg(feature = "client")]
use reqwest::Client as HttpClient;
#[cfg(feature = "client")]
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum CbrApiError {
    #[cfg(feature = "client")]
    #[error("HTTP Error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("Rate Limit Exceeded (429)")]
    RateLimited,
    #[error("Service Unavailable (503)")]
    Unavailable,
    #[error("API JSON Parse Error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Authentication Error: {0}")]
    Auth(alloc::string::String),
}

#[cfg(feature = "client")]
pub struct CbrApiClient {
    http: HttpClient,
    base_url: String,
    api_key: String,
}

#[cfg(feature = "client")]
impl CbrApiClient {
    pub fn builder() -> CbrApiBuilder {
        CbrApiBuilder::default()
    }

    pub fn antifraud(&self) -> antifraud::AntiFraudClient<'_> {
        antifraud::AntiFraudClient::new(self)
    }

    pub fn tspi(&self) -> tspi::TspiClient<'_> {
        tspi::TspiClient::new(self)
    }

    pub fn ebs(&self) -> ebs::EbsClient<'_> {
        ebs::EbsClient::new(self)
    }
}

#[cfg(feature = "client")]
#[derive(Default)]
pub struct CbrApiBuilder {
    base_url: Option<String>,
    api_key: Option<String>,
}

#[cfg(feature = "client")]
impl CbrApiBuilder {
    pub fn base_url(mut self, url: impl Into<String>) -> Self {
        self.base_url = Some(url.into());
        self
    }

    pub fn api_key(mut self, key: impl Into<String>) -> Self {
        self.api_key = Some(key.into());
        self
    }

    pub fn build(self) -> Result<CbrApiClient, CbrApiError> {
        let base_url = self
            .base_url
            .unwrap_or_else(|| "https://finapi.cbr.ru".to_string());
        let api_key = self
            .api_key
            .ok_or_else(|| CbrApiError::Auth("Missing API Key".into()))?;

        let http = HttpClient::builder()
            .timeout(Duration::from_secs(10))
            .build()?;

        Ok(CbrApiClient {
            http,
            base_url,
            api_key,
        })
    }
}
