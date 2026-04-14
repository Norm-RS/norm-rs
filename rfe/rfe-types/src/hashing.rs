use crate::blake3_hash;
use alloc::string::ToString;
use alloc::vec::Vec;

#[derive(Debug, thiserror::Error)]
pub enum CanonicalHashError {
    #[error("canonical serialization failed")]
    SerializationFailed,
}

/// Canonicalize reports into a deterministic byte stream (compact JSON).
///
/// Diagnostic helper for inspection/debugging only.
/// For authoritative Seal v1 results hash, use `hash_canonical_reports`.
pub fn canonical_json(
    reports: &mut [crate::ComplianceReport],
) -> Result<Vec<u8>, CanonicalHashError> {
    // 1. Deterministic sort by transaction_id
    reports.sort();

    // 2. Serialize to compact JSON
    // Note: rfe-types ensures decimals are used instead of floats.
    serde_json::to_vec(reports).map_err(|_| CanonicalHashError::SerializationFailed)
}

/// Helper to compute canonical result hash for Seal v1.
///
/// Sorts `reports` in-place by `transaction_id` before hashing.
pub fn hash_canonical_reports(
    reports: &mut [crate::ComplianceReport],
) -> Result<[u8; 32], CanonicalHashError> {
    reports.sort();

    let mut hasher = blake3::Hasher::new();
    hasher.update(b"[");
    for (idx, report) in reports.iter().enumerate() {
        if idx > 0 {
            hasher.update(b",");
        }
        hash_report_canonical(&mut hasher, report);
    }
    hasher.update(b"]");
    Ok(*hasher.finalize().as_bytes())
}

/// Helper to hash a raw request byte stream (Lockstep Invariant).
pub fn hash_raw_request(data: &[u8]) -> [u8; 32] {
    blake3_hash(data)
}

fn hash_report_canonical(hasher: &mut blake3::Hasher, report: &crate::ComplianceReport) {
    hasher.update(b"{\"transaction_id\":\"");
    hash_json_escaped(hasher, report.transaction_id.as_bytes());
    hasher.update(b"\",\"pdn_ratio\":\"");
    hasher.update(report.pdn_ratio.to_string().as_bytes());
    hasher.update(b"\",\"is_pdn_risky\":");
    if report.is_pdn_risky {
        hasher.update(b"true");
    } else {
        hasher.update(b"false");
    }
    hasher.update(b",\"fraud_signs\":[");
    for (idx, sign) in report.fraud_signs.iter().enumerate() {
        if idx > 0 {
            hasher.update(b",");
        }
        hasher.update(b"\"");
        match sign {
            crate::FraudSign::ReceiverInDatabase => {
                hasher.update(b"ReceiverInDatabase");
            }
            crate::FraudSign::DeviceInDatabase => {
                hasher.update(b"DeviceInDatabase");
            }
            crate::FraudSign::AtypicalTransaction => {
                hasher.update(b"AtypicalTransaction");
            }
            crate::FraudSign::SuspiciousSbpTransfer => {
                hasher.update(b"SuspiciousSbpTransfer");
            }
            crate::FraudSign::SuspiciousNfcActivity => {
                hasher.update(b"SuspiciousNfcActivity");
            }
            crate::FraudSign::MultipleAccountsFromSingleDevice => {
                hasher.update(b"MultipleAccountsFromSingleDevice");
            }
            crate::FraudSign::InconsistentGeolocation => {
                hasher.update(b"InconsistentGeolocation");
            }
            crate::FraudSign::HighVelocityTransfersInShortWindow => {
                hasher.update(b"HighVelocityTransfersInShortWindow");
            }
            crate::FraudSign::RemoteAccessToolDetected => {
                hasher.update(b"RemoteAccessToolDetected");
            }
            crate::FraudSign::KnownProxyOrVpnEndpoint => {
                hasher.update(b"KnownProxyOrVpnEndpoint");
            }
            crate::FraudSign::SocialEngineeringPatternDetected => {
                hasher.update(b"SocialEngineeringPatternDetected");
            }
            crate::FraudSign::ExternalOperatorSignal => {
                hasher.update(b"ExternalOperatorSignal");
            }
            crate::FraudSign::Other(v) => {
                hash_json_escaped(hasher, v.as_bytes());
            }
        }
        hasher.update(b"\"");
    }
    hasher.update(b"],\"recommendation\":\"");
    hash_json_escaped(hasher, report.recommendation.as_bytes());
    hasher.update(b"\",\"created_at_micros\":");
    hasher.update(report.created_at_micros.to_string().as_bytes());
    hasher.update(b"}");
}

fn hash_json_escaped(hasher: &mut blake3::Hasher, bytes: &[u8]) {
    for &b in bytes {
        match b {
            b'"' => {
                hasher.update(b"\\\"");
            }
            b'\\' => {
                hasher.update(b"\\\\");
            }
            b'\n' => {
                hasher.update(b"\\n");
            }
            b'\r' => {
                hasher.update(b"\\r");
            }
            b'\t' => {
                hasher.update(b"\\t");
            }
            0x00..=0x1F => {
                let mut esc: [u8; 6] = [b'\\', b'u', b'0', b'0', b'0', b'0'];
                let hi = (b >> 4) & 0x0F;
                let lo = b & 0x0F;
                esc[4] = if hi < 10 { b'0' + hi } else { b'a' + (hi - 10) };
                esc[5] = if lo < 10 { b'0' + lo } else { b'a' + (lo - 10) };
                hasher.update(&esc);
            }
            _ => {
                hasher.update(&[b]);
            }
        }
    }
}
