//! Shared types for Rust Fintech Ecosystem (RFE).
//!
//! Provides `newtype` identifiers, Blake3 hashing helpers, and `Decimal`
//! re-exports used across RFE crates. No I/O, no async.

#![no_std]

extern crate alloc;
use alloc::string::String;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub use rust_decimal;

pub mod hashing;
pub mod test_vectors;

// ---- Newtype IDs -------------------------------------------------------

/// Wrapper to prevent PII leakage in logs.
#[derive(Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Sensitive<T>(pub T);

impl<T> core::fmt::Debug for Sensitive<T> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "[REDACTED PII]")
    }
}

/// Russian taxpayer identification number (INN), 10 or 12 digits.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Inn(pub Sensitive<String>);

impl Inn {
    /// Create without validation (use `parse` for validated construction).
    pub fn new_unchecked(s: impl Into<String>) -> Self {
        Self(Sensitive(s.into()))
    }

    /// Validates INN length (10 for legal entities, 12 for individuals).
    pub fn parse(s: impl Into<String>) -> Result<Self, RfeError> {
        let s = s.into();
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() == 10 || digits.len() == 12 {
            Ok(Self(Sensitive(digits)))
        } else {
            Err(RfeError::InvalidInn(s))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0 .0
    }
}

/// Russian OGRN — primary state registration number, 13 or 15 digits.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Ogrn(pub Sensitive<String>);

impl Ogrn {
    pub fn parse(s: impl Into<String>) -> Result<Self, RfeError> {
        let s = s.into();
        let digits: String = s.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.len() == 13 || digits.len() == 15 {
            Ok(Self(Sensitive(digits)))
        } else {
            Err(RfeError::InvalidOgrn(s))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0 .0
    }
}

/// Unique request/correlation identifier (UUID v4).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RequestId(Uuid);

impl RequestId {
    #[cfg(feature = "v4")]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Default for RequestId {
    fn default() -> Self {
        Self::nil()
    }
}

impl core::fmt::Display for RequestId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Loan/order identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LoanId(Uuid);

impl LoanId {
    #[cfg(feature = "v4")]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Default for LoanId {
    fn default() -> Self {
        Self::nil()
    }
}

impl core::fmt::Display for LoanId {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Client/customer identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ClientId(Uuid);

impl ClientId {
    #[cfg(feature = "v4")]
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn from_uuid(uuid: Uuid) -> Self {
        Self(uuid)
    }

    pub fn nil() -> Self {
        Self(Uuid::nil())
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }
}

impl Default for ClientId {
    fn default() -> Self {
        Self::nil()
    }
}

// ---- Blake3 Hashing ----------------------------------------------------

/// Compute blake3 hash of a single byte slice.
pub fn blake3_hash(data: &[u8]) -> [u8; 32] {
    *blake3::hash(data).as_bytes()
}

/// Iterated blake3 hash over multiple byte slices (in order).
/// Matches the pattern from zerocore-gateway/common for audit chaining.
pub fn blake3_chain(parts: &[&[u8]]) -> [u8; 32] {
    let mut hasher = blake3::Hasher::new();
    for part in parts {
        hasher.update(part);
    }
    *hasher.finalize().as_bytes()
}

/// Any type that can produce a deterministic content hash.
pub trait Hashable {
    fn content_hash(&self) -> [u8; 32];
}

// ---- Audit Entry -------------------------------------------------------

/// Append-only audit entry — blake3 chained for tamper evidence.
/// Clean-room of LedgerEntry from zerocore-gateway/audit-ledger.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub request_id: RequestId,
    pub parent_hash: [u8; 32],
    pub payload_hash: [u8; 32],
    pub entry_hash: [u8; 32],
    pub seal: [u8; 32],
    pub created_at_micros: u64,
    pub processing_time_micros: u64,
    /// Operator identity binding hash (deterministic, non-PII).
    pub operator_binding_hash: [u8; 32],
    /// Progressive Attestation: operator session binding.
    pub session_nonce: [u8; 32],
}

/// NORM Protocol v1 Seal Input.
/// Prefix: "NORM_SEAL_V1" (12 bytes)
pub const SEAL_DOMAIN_PREFIX: &[u8; 12] = b"NORM_SEAL_V1";

/// Frozen in `rfe-types` v0.1.0 for TrustBox compatibility.
/// Breaking field changes require a semver-major release.
/// Canonical fields: `nonce`, `request_hash`, `result_hash`, `chain_head_pre`.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SealInput {
    pub version: u32,
    pub nonce: [u8; 32],
    pub request_hash: [u8; 32],
    pub result_hash: [u8; 32],
    pub chain_head_pre: [u8; 32],
}

impl SealInput {
    pub fn new_v1(
        nonce: [u8; 32],
        request_hash: [u8; 32],
        result_hash: [u8; 32],
        chain_head_pre: [u8; 32],
    ) -> Self {
        Self {
            version: 1,
            nonce,
            request_hash,
            result_hash,
            chain_head_pre,
        }
    }

    pub fn compute_seal(&self) -> [u8; 32] {
        blake3_chain(&[
            SEAL_DOMAIN_PREFIX,
            &self.version.to_le_bytes(),
            &self.nonce,
            &self.request_hash,
            &self.result_hash,
            &self.chain_head_pre,
        ])
    }
}

impl AuditEntry {
    pub fn genesis(timestamp_micros: u64, payload: &[u8], nonce: Option<&[u8; 32]>) -> Self {
        let parent_hash = [0u8; 32];
        let payload_hash = blake3_hash(payload);
        let nonce_val = nonce.cloned().unwrap_or([0u8; 32]);

        // Protocol v1: Evolution = blake3(head_pre || req_hash || res_hash || seal)
        // For genesis, seal is computed from provided nonce and payload.
        let seal_input = SealInput::new_v1(
            nonce_val,
            payload_hash,
            payload_hash, // Simplified for legacy genesis; in actual TB it binds results
            parent_hash,
        );
        let seal = seal_input.compute_seal();
        let entry_hash = blake3_chain(&[&parent_hash, &payload_hash, &payload_hash, &seal]);

        Self {
            request_id: RequestId::nil(),
            parent_hash,
            payload_hash,
            entry_hash,
            seal,
            created_at_micros: timestamp_micros,
            processing_time_micros: 0,
            operator_binding_hash: [0u8; 32],
            session_nonce: nonce_val,
        }
    }

    pub fn next(&self, timestamp_micros: u64, payload: &[u8], nonce: Option<&[u8; 32]>) -> Self {
        let payload_hash = blake3_hash(payload);
        let nonce_val = nonce.cloned().unwrap_or(self.session_nonce);

        // Protocol v1 evolution: next_head = blake3(head_pre || req_hash || res_hash || seal)
        let seal_input = SealInput::new_v1(
            nonce_val,
            payload_hash,
            payload_hash, // Placeholder
            self.entry_hash,
        );
        let seal = seal_input.compute_seal();

        let entry_hash = blake3_chain(&[&self.entry_hash, &payload_hash, &payload_hash, &seal]);

        Self {
            request_id: RequestId::nil(),
            parent_hash: self.entry_hash,
            payload_hash,
            entry_hash,
            seal,
            created_at_micros: timestamp_micros,
            processing_time_micros: 0,
            operator_binding_hash: self.operator_binding_hash,
            session_nonce: nonce_val,
        }
    }

    pub fn verify_chain(&self, parent: &AuditEntry) -> bool {
        self.parent_hash == parent.entry_hash
    }

    pub fn hash_hex(&self) -> String {
        let mut s = String::with_capacity(64);
        for byte in self.entry_hash {
            let _ = core::fmt::write(&mut s, format_args!("{:02x}", byte));
        }
        s
    }
}

// ---- Canonical Protocol Types ------------------------------------------

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, PartialOrd, Ord)]
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
    /// Signal received from another payment system operator (OD-2506 sign 12).
    ExternalOperatorSignal,
    Other(String),
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ComplianceReport {
    pub transaction_id: String,
    pub pdn_ratio: Decimal,
    pub is_pdn_risky: bool,
    pub fraud_signs: alloc::vec::Vec<FraudSign>,
    pub recommendation: String,
    pub created_at_micros: u64,
}

impl PartialOrd for ComplianceReport {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ComplianceReport {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        self.transaction_id.cmp(&other.transaction_id)
    }
}

// ---- GOST Export (optional) --------------------------------------------

/// Dual-hash audit root for regulatory export (GOST R 34.11-2012 / Streebog-256).
/// The blake3_root is the internal chain root; streebog_root is its Streebog-256 re-hash
/// required for CBR submissions and SMEV 4 payload signatures.
#[cfg(feature = "gost-export")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportableAuditRoot {
    /// Internal BLAKE3 chain root (high-speed, no_std).
    pub blake3_root: [u8; 32],
    /// GOST R 34.11-2012 (Streebog-256) re-hash of blake3_root for regulatory submissions.
    pub streebog_root: [u8; 32],
}

#[cfg(feature = "gost-export")]
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum AuditSignatureAlgorithm {
    Blake3DetachedV1,
}

#[cfg(feature = "gost-export")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignedAuditExportEnvelope {
    pub algorithm: AuditSignatureAlgorithm,
    pub entry_hash: [u8; 32],
    pub seal: [u8; 32],
    pub blake3_root: [u8; 32],
    pub streebog_root: [u8; 32],
    pub operator_binding_hash: [u8; 32],
    pub session_nonce: [u8; 32],
    pub processing_time_micros: u64,
    pub signature: [u8; 32],
}

#[cfg(feature = "gost-export")]
#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
pub enum AuditExportError {
    #[error("invalid audit entry hash")]
    InvalidEntryHash,
    #[error("invalid seal")]
    InvalidSeal,
}

/// Produce an ExportableAuditRoot from an existing BLAKE3 chain root.
/// This is the only function in the codebase that computes Streebog-256.
/// Call exclusively at the export boundary (SMEV 4 payload, CBR submission header).
#[cfg(feature = "gost-export")]
pub fn audit_root_gost_export(blake3_root: &[u8; 32]) -> ExportableAuditRoot {
    use streebog::{Digest, Streebog256};
    let streebog_bytes: [u8; 32] = Streebog256::digest(blake3_root).into();
    ExportableAuditRoot {
        blake3_root: *blake3_root,
        streebog_root: streebog_bytes,
    }
}

#[cfg(feature = "gost-export")]
pub fn build_signed_audit_export(
    entry: &AuditEntry,
) -> Result<SignedAuditExportEnvelope, AuditExportError> {
    if entry.entry_hash == [0u8; 32] {
        return Err(AuditExportError::InvalidEntryHash);
    }
    if entry.seal == [0u8; 32] {
        return Err(AuditExportError::InvalidSeal);
    }

    let roots = audit_root_gost_export(&entry.entry_hash);
    let signature = blake3_chain(&[
        b"NORM_EXPORT_SIG_V1",
        &entry.entry_hash,
        &entry.seal,
        &roots.streebog_root,
        &entry.operator_binding_hash,
        &entry.session_nonce,
        &entry.processing_time_micros.to_le_bytes(),
    ]);

    Ok(SignedAuditExportEnvelope {
        algorithm: AuditSignatureAlgorithm::Blake3DetachedV1,
        entry_hash: entry.entry_hash,
        seal: entry.seal,
        blake3_root: roots.blake3_root,
        streebog_root: roots.streebog_root,
        operator_binding_hash: entry.operator_binding_hash,
        session_nonce: entry.session_nonce,
        processing_time_micros: entry.processing_time_micros,
        signature,
    })
}

/// Hex-encode a 32-byte hash to a stack-allocated String (no_std compatible).
#[cfg(feature = "gost-export")]
impl ExportableAuditRoot {
    pub fn streebog_hex(&self) -> String {
        let mut s = String::with_capacity(64);
        for byte in self.streebog_root {
            let _ = core::fmt::write(&mut s, format_args!("{:02x}", byte));
        }
        s
    }
}

// ---- Decimal Utilities -------------------------------------------------

/// Round to 2 decimal places using banker's rounding (HALF_EVEN).
/// Used for financial calculations per CBR requirements.
pub fn round_financial(d: Decimal) -> Decimal {
    d.round_dp(2)
}

/// Safe division returning zero instead of panicking on zero denominator.
pub fn safe_div(numerator: Decimal, denominator: Decimal) -> Decimal {
    if denominator.is_zero() {
        Decimal::ZERO
    } else {
        numerator / denominator
    }
}

// ---- Errors ------------------------------------------------------------

#[derive(Debug, thiserror::Error)]
pub enum RfeError {
    #[error("Invalid INN format: {0}")]
    InvalidInn(String),
    #[error("Invalid OGRN format: {0}")]
    InvalidOgrn(String),
    #[error("Decimal parse error: {0}")]
    DecimalParse(String),
}

// ---- Tests -------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alloc::string::ToString;

    #[test]
    fn inn_valid_10_digit() {
        let inn = Inn::parse("7700000000").unwrap();
        assert_eq!(inn.as_str(), "7700000000");
    }

    #[test]
    fn inn_valid_12_digit() {
        let inn = Inn::parse("770000000001").unwrap();
        assert_eq!(inn.as_str(), "770000000001");
    }

    #[test]
    fn inn_invalid_rejects() {
        assert!(Inn::parse("123").is_err());
        assert!(Inn::parse("12345678901234").is_err());
    }

    #[test]
    fn ogrn_valid_13() {
        assert!(Ogrn::parse("1027700000")
            .or(Ogrn::parse("1027700000001"))
            .is_ok());
    }

    #[test]
    fn blake3_chain_deterministic() {
        let h1 = blake3_chain(&[b"hello", b"world"]);
        let h2 = blake3_chain(&[b"hello", b"world"]);
        assert_eq!(h1, h2);
        let h3 = blake3_chain(&[b"world", b"hello"]);
        assert_ne!(h1, h3);
    }

    #[test]
    fn audit_chain_verifies() {
        let genesis = AuditEntry::genesis(0, b"genesis payload", None);
        let next = genesis.next(100, b"test payload", None);
        assert!(next.verify_chain(&genesis));
        // Tampered parent fails
        let mut tampered = next.clone();
        tampered.parent_hash = [0xff; 32];
        assert!(!tampered.verify_chain(&genesis));
    }

    #[test]
    fn safe_div_zero_denominator() {
        assert_eq!(safe_div(Decimal::new(100, 0), Decimal::ZERO), Decimal::ZERO);
    }

    #[test]
    fn round_financial_banker() {
        // 1.005 rounds to 1.00 with HALF_EVEN (banker's rounding)
        let d = Decimal::new(1005, 3); // 1.005
        let rounded = round_financial(d);
        assert_eq!(rounded.to_string(), "1.00");
    }
}
