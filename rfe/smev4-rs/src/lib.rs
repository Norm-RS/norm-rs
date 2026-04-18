#![no_std]
//! Async SMEV 4 client primitives for Russian inter-agency integrations.
//!
//! SMEV 4 interaction model is queue-based:
//! 1. Submit request
//! 2. Receive queue ticket (`QueueTicket`)
//! 3. Poll response or handle callback

extern crate alloc;
#[cfg(feature = "std")]
extern crate std;

#[cfg(feature = "std")]
pub mod client;
pub mod rsocket;
#[cfg(feature = "std")]
pub mod services;

#[cfg(feature = "std")]
pub use client::{AuthProvider, PollConfig, QueueTicket, SmevClient};
pub use rsocket::{RSocketClient, RSocketFrame};

#[derive(Debug, thiserror::Error)]
pub enum SmevError {
    #[cfg(feature = "std")]
    #[error("HTTP error: {0}")]
    Http(reqwest::Error),
    #[cfg(feature = "std")]
    #[error("XML parse error: {0}")]
    Xml(quick_xml::Error),
    #[error("Authentication/configuration error: {0}")]
    Auth(alloc::string::String),
    #[error("Timeout while polling SMEV queue")]
    Timeout,
    #[error("SMEV service unavailable: {reason}")]
    Unavailable { reason: alloc::string::String },
    #[error("Payload/response error: {0}")]
    Payload(alloc::string::String),
}

#[cfg(feature = "std")]
impl From<reqwest::Error> for SmevError {
    fn from(e: reqwest::Error) -> Self {
        Self::Http(e)
    }
}

#[cfg(feature = "std")]
impl From<quick_xml::Error> for SmevError {
    fn from(e: quick_xml::Error) -> Self {
        Self::Xml(e)
    }
}
