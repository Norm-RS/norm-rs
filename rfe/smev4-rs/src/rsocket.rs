//! SMEV 4 transport primitives (Phase B simplified internal framing).
//!
//! NOTE: Frame encoding in this module is a simplified internal format and is
//! not wire-compatible with the official RSocket binary protocol (rsocket.io).
//! Full wire-compatible framing is planned for a later wave.

use alloc::string::String;
use alloc::vec::Vec;
use core::convert::Into;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RSocketFrame {
    RequestResponse { id: u32, payload: Vec<u8> },
    Stream { id: u32, payload: Vec<u8> },
    Cancel { id: u32 },
    Error { id: u32, message: String },
}

impl RSocketFrame {
    /// Encodes the frame into a binary buffer for SMEV 4 transport.
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        match self {
            Self::RequestResponse { id, payload } => {
                buf.push(0x01); // Type
                buf.extend_from_slice(&id.to_be_bytes());
                buf.extend_from_slice(payload);
            }
            Self::Stream { id, payload } => {
                buf.push(0x02); // Type
                buf.extend_from_slice(&id.to_be_bytes());
                buf.extend_from_slice(payload);
            }
            Self::Cancel { id } => {
                buf.push(0x09);
                buf.extend_from_slice(&id.to_be_bytes());
            }
            Self::Error { id, message } => {
                buf.push(0x0B);
                buf.extend_from_slice(&id.to_be_bytes());
                buf.extend_from_slice(message.as_bytes());
            }
        }
        buf
    }
}

pub struct RSocketClient {
    pub endpoint: String,
}

impl RSocketClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            endpoint: endpoint.into(),
        }
    }

    /// Optimized SMEV 4 audit stream push with binary framing.
    pub async fn push_audit_frame(&self, frame: RSocketFrame) -> Result<(), crate::SmevError> {
        let _encoded = frame.encode();
        // In a real SMEV 4 environment, this would write to a TCP/TLS stream.
        // For Phase B, we log the binary commitment.
        #[cfg(feature = "std")]
        std::println!(
            "[rsocket] SMEV 4 Commit: {} bytes to {}",
            _encoded.len(),
            self.endpoint
        );
        Ok(())
    }
}
