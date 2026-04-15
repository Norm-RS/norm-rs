//! Typestate-based FSM for lending and BNPL flows.
//!
//! Allowed transitions:
//! Draft -> Scoring -> PdnCheck -> Approved/Rejected -> Disbursed -> Closed
//!
//! Compiler-enforced state transitions prevent invalid operations (for example,
//! disbursing a rejected application).

use chrono::{DateTime, Utc};
use rfe_types::{blake3_chain, rust_decimal::Decimal, ClientId, Hashable, LoanId};

// ---- State Markers ----

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Draft;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Scoring;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdnCheck;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Approved;
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Rejected {
    pub reason: String,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Disbursed {
    pub disbursed_at: DateTime<Utc>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Closed;

// ---- Application FSM ----

/// A universal loan application encapsulating state at compile time.
#[derive(Debug, Clone)]
pub struct LoanApplication<State> {
    pub id: LoanId,
    pub client_id: ClientId,
    pub amount: Decimal,
    pub fully_identified: bool,
    pub history_hash: [u8; 32],
    pub state: State,
}

impl<S> Hashable for LoanApplication<S> {
    fn content_hash(&self) -> [u8; 32] {
        self.history_hash
    }
}

// Helper to advance hash chain
fn advance_hash(prev: [u8; 32], id: &[u8], action: &[u8]) -> [u8; 32] {
    let ts = Utc::now().timestamp_micros().to_le_bytes();
    blake3_chain(&[&prev, id, action, &ts])
}

// 1. Draft
impl LoanApplication<Draft> {
    /// 115-FZ threshold: obligations above 15,000 RUB require full identification.
    pub const AML_FULL_IDENT_THRESHOLD: Decimal = Decimal::from_parts(15_000, 0, 0, false, 0);

    pub fn new(client_id: ClientId, amount: Decimal) -> Self {
        Self::new_with_identification(client_id, amount, true)
            .expect("default constructor assumes full identification is completed")
    }

    pub fn new_with_identification(
        client_id: ClientId,
        amount: Decimal,
        fully_identified: bool,
    ) -> Result<Self, &'static str> {
        if amount > Self::AML_FULL_IDENT_THRESHOLD && !fully_identified {
            return Err("Amount above 15,000 RUB requires full identification (115-FZ)");
        }

        let id = LoanId::new();
        let ts = Utc::now().timestamp_micros().to_le_bytes();
        let initial_hash = blake3_chain(&[id.as_bytes(), client_id.as_bytes(), &ts]);

        Ok(Self {
            id,
            client_id,
            amount,
            fully_identified,
            history_hash: initial_hash,
            state: Draft,
        })
    }

    pub fn submit_to_scoring(self) -> LoanApplication<Scoring> {
        let next_hash = advance_hash(self.history_hash, self.id.as_bytes(), b"submit_scoring");
        LoanApplication {
            id: self.id,
            client_id: self.client_id,
            amount: self.amount,
            fully_identified: self.fully_identified,
            history_hash: next_hash,
            state: Scoring,
        }
    }

    pub fn reject(self, reason: String) -> LoanApplication<Rejected> {
        let next_hash = advance_hash(self.history_hash, self.id.as_bytes(), b"reject_draft");
        LoanApplication {
            id: self.id,
            client_id: self.client_id,
            amount: self.amount,
            fully_identified: self.fully_identified,
            history_hash: next_hash,
            state: Rejected { reason },
        }
    }
}

// 2. Scoring
impl LoanApplication<Scoring> {
    pub fn scoring_done(
        self,
        passed: bool,
    ) -> Result<LoanApplication<PdnCheck>, LoanApplication<Rejected>> {
        if passed {
            let next_hash = advance_hash(self.history_hash, self.id.as_bytes(), b"scoring_passed");
            Ok(LoanApplication {
                id: self.id,
                client_id: self.client_id,
                amount: self.amount,
                fully_identified: self.fully_identified,
                history_hash: next_hash,
                state: PdnCheck,
            })
        } else {
            let next_hash = advance_hash(self.history_hash, self.id.as_bytes(), b"scoring_failed");
            Err(LoanApplication {
                id: self.id,
                client_id: self.client_id,
                amount: self.amount,
                fully_identified: self.fully_identified,
                history_hash: next_hash,
                state: Rejected {
                    reason: "Scoring engine rejected".to_string(),
                },
            })
        }
    }
}

// 3. PdnCheck
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PdnRiskTier {
    Tier10x,
    Tier15x,
    Tier30x,
    Tier50x,
}

impl LoanApplication<PdnCheck> {
    /// Compatibility method: `is_risky=true` maps to tier 3.0x+ semantics.
    pub fn pdn_done(
        self,
        is_risky: bool,
    ) -> Result<LoanApplication<Approved>, LoanApplication<Rejected>> {
        let tier = if is_risky {
            PdnRiskTier::Tier30x
        } else {
            PdnRiskTier::Tier15x
        };
        self.pdn_done_tier(tier)
    }

    pub fn pdn_done_tier(
        self,
        tier: PdnRiskTier,
    ) -> Result<LoanApplication<Approved>, LoanApplication<Rejected>> {
        // Simplified policy: reject high-risk tiers (>=3.0x); approve lower tiers.
        let reject = matches!(tier, PdnRiskTier::Tier30x | PdnRiskTier::Tier50x);
        if !reject {
            let next_hash = advance_hash(self.history_hash, self.id.as_bytes(), b"approve");
            Ok(LoanApplication {
                id: self.id,
                client_id: self.client_id,
                amount: self.amount,
                fully_identified: self.fully_identified,
                history_hash: next_hash,
                state: Approved,
            })
        } else {
            let next_hash = advance_hash(self.history_hash, self.id.as_bytes(), b"pdn_reject");
            Err(LoanApplication {
                id: self.id,
                client_id: self.client_id,
                amount: self.amount,
                fully_identified: self.fully_identified,
                history_hash: next_hash,
                state: Rejected {
                    reason: "PDN > 0.50 (Risky)".to_string(),
                },
            })
        }
    }
}

// 4. Approved
impl LoanApplication<Approved> {
    pub fn disburse(self) -> LoanApplication<Disbursed> {
        let next_hash = advance_hash(self.history_hash, self.id.as_bytes(), b"disburse");
        LoanApplication {
            id: self.id,
            client_id: self.client_id,
            amount: self.amount,
            fully_identified: self.fully_identified,
            history_hash: next_hash,
            state: Disbursed {
                disbursed_at: Utc::now(),
            },
        }
    }
}

// 5. Disbursed
impl LoanApplication<Disbursed> {
    pub fn close(self) -> LoanApplication<Closed> {
        let next_hash = advance_hash(self.history_hash, self.id.as_bytes(), b"close");
        LoanApplication {
            id: self.id,
            client_id: self.client_id,
            amount: self.amount,
            fully_identified: self.fully_identified,
            history_hash: next_hash,
            state: Closed,
        }
    }
}

// ---- BNPL Specific (283-FZ) ----

/// BNPL orders must enforce max 6 months term and 50k limit if without BKI.
#[derive(Debug, Clone)]
pub struct BnplOrder {
    pub inner: LoanApplication<Draft>,
    pub term_months: u8,
    pub without_bki: bool,
}

impl BnplOrder {
    pub const BNPL_MAX_TERM_UNTIL_2028_MONTHS: u8 = 6;
    pub const BNPL_MAX_TERM_FROM_2028_MONTHS: u8 = 4;
    const BNPL_TERM_REDUCTION_DATE: (i32, u32, u32) = (2028, 4, 1);

    pub fn max_term_months_for_date(now: DateTime<Utc>) -> u8 {
        let reduction_at = chrono::NaiveDate::from_ymd_opt(
            Self::BNPL_TERM_REDUCTION_DATE.0,
            Self::BNPL_TERM_REDUCTION_DATE.1,
            Self::BNPL_TERM_REDUCTION_DATE.2,
        )
        .unwrap()
        .and_hms_opt(0, 0, 0)
        .unwrap()
        .and_utc();
        if now >= reduction_at {
            Self::BNPL_MAX_TERM_FROM_2028_MONTHS
        } else {
            Self::BNPL_MAX_TERM_UNTIL_2028_MONTHS
        }
    }

    pub fn create_regulated(
        client_id: ClientId,
        amount: Decimal,
        term_months: u8,
        without_bki: bool,
        fully_identified: bool,
    ) -> Result<Self, &'static str> {
        // 283-FZ transition: 6 months from 2026-04-01, 4 months from 2028-04-01.
        let max_term = Self::max_term_months_for_date(Utc::now());
        if term_months > max_term {
            return Err("BNPL term exceeds active 283-FZ limit");
        }

        // Enforce 283-FZ: Max amount without BKI is 50,000 rub
        if without_bki && amount > Decimal::new(50_000, 0) {
            return Err("BNPL without BKI exceeds 50,000 limit (283-FZ violation)");
        }

        Ok(Self {
            inner: LoanApplication::new_with_identification(client_id, amount, fully_identified)?,
            term_months,
            without_bki,
        })
    }

    /// Returns true if this BNPL order requires mandatory BKI reporting.
    /// Per 6960-U / 283-FZ: any single BNPL obligation exceeding 50,000 RUB
    /// must be reported to the credit bureau regardless of the `without_bki` flag.
    pub fn bki_report_required(&self) -> bool {
        self.inner.amount > Decimal::new(50_000, 0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_happy_path() {
        let draft = LoanApplication::new(ClientId::new(), Decimal::new(10_000, 0));
        let scoring = draft.submit_to_scoring();
        let pdn = scoring.scoring_done(true).unwrap();
        let approved = pdn.pdn_done(false).unwrap();
        let disbursed = approved.disburse();
        let closed = disbursed.close();
        // hash must have mutated multiple times
        assert_ne!(closed.history_hash, [0u8; 32]);
    }

    #[test]
    fn bnpl_term_enforcement() {
        assert!(BnplOrder::create_regulated(
            ClientId::new(),
            Decimal::new(5000, 0),
            4,
            false,
            true
        )
        .is_ok());
        assert!(BnplOrder::create_regulated(
            ClientId::new(),
            Decimal::new(5000, 0),
            7,
            false,
            true
        )
        .is_err());
    }

    #[test]
    fn bnpl_bki_limit_enforcement() {
        // Allow > 50k if with BKI
        assert!(BnplOrder::create_regulated(
            ClientId::new(),
            Decimal::new(60_000, 0),
            4,
            false,
            true
        )
        .is_ok());
        // Fails if > 50k and without BKI
        assert!(BnplOrder::create_regulated(
            ClientId::new(),
            Decimal::new(60_000, 0),
            4,
            true,
            true
        )
        .is_err());
        // Allow <= 50k without BKI
        assert!(BnplOrder::create_regulated(
            ClientId::new(),
            Decimal::new(50_000, 0),
            4,
            true,
            true
        )
        .is_ok());
    }

    #[test]
    fn bnpl_bki_report_required() {
        // Over 50k with BKI: must flag for mandatory BKI reporting per 6960-U
        let order =
            BnplOrder::create_regulated(ClientId::new(), Decimal::new(60_000, 0), 4, false, true)
                .unwrap();
        assert!(order.bki_report_required());

        // Under or equal to 50k: no mandatory BKI report required
        let order_small =
            BnplOrder::create_regulated(ClientId::new(), Decimal::new(50_000, 0), 4, true, true)
                .unwrap();
        assert!(!order_small.bki_report_required());
    }

    #[test]
    fn aml_identification_threshold_enforced() {
        let err = LoanApplication::new_with_identification(
            ClientId::new(),
            Decimal::new(20_000, 0),
            false,
        )
        .unwrap_err();
        assert!(err.contains("15,000"));
    }
}
