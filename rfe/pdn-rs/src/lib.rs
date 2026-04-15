//! PDN (Debt Burden Ratio) calculation per CBR Directive 6960-U
//! (16.12.2024, as amended by 7226-U).
//! Pure Rust implementation, `no_std` compatible, suitable for embedded scoring engines.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;
use rfe_types::{round_financial, rust_decimal::Decimal, safe_div};

/// A single loan/credit card obligation for a borrower.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Obligation {
    /// Scheduled monthly payment.
    pub monthly_payment: Decimal,
    /// Whether this is a credit card (affects calculation logic).
    pub is_credit_card: bool,
    /// Credit limit (only required for credit cards).
    pub credit_limit: Option<Decimal>,
}

impl Obligation {
    /// Calculate the recognized payment amount for PDN purposes.
    /// Per 6960-U/7226-U, credit cards assume 5% of the credit limit if actual payment is unknown.
    pub fn recognized_payment(&self) -> Decimal {
        if self.is_credit_card {
            if let Some(limit) = self.credit_limit {
                // 5% of credit limit
                let five_percent = limit * Decimal::new(5, 2);
                core::cmp::max(self.monthly_payment, five_percent)
            } else {
                self.monthly_payment
            }
        } else {
            self.monthly_payment
        }
    }
}

/// Source of income for PDN calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum IncomeSource {
    /// Confirmed via FNS (SMEV) or PFR. The gold standard.
    Confirmed(Decimal),
    /// Declared by the borrower (high risk).
    Declared(Decimal),
    /// Estimated based on regional/industry averages.
    /// Not recognized as confirmed income under 6960-U / 7226-U.
    /// Treated as unconfirmed for capital policy purposes.
    Estimated(Decimal),
    /// Family total income (Draft 2026 extension for large mortgages).
    FamilyTotal(Decimal),
}

impl IncomeSource {
    pub fn value(&self) -> Decimal {
        match self {
            Self::Confirmed(v) | Self::Declared(v) | Self::Estimated(v) | Self::FamilyTotal(v) => {
                *v
            }
        }
    }

    /// Does the regulator consider this confirmed?
    pub fn is_confirmed(&self) -> bool {
        matches!(self, Self::Confirmed(_) | Self::FamilyTotal(_))
    }
}

/// The result of a PDN calculation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PdnResult {
    /// The computed ratio (e.g., 0.45 for 45%).
    pub ratio: Decimal,
    /// Total recognized monthly payments across all obligations.
    pub total_payments: Decimal,
    /// True if ratio exceeds 0.50.
    /// Simplified risk flag only. Capital tiers are:
    /// 1.0x (<=30%), 1.5x (31-50%), 3.0x (51-70%), 5.0x (>70% or unconfirmed income).
    pub is_risky: bool,
}

pub struct PdnCalculator;

impl PdnCalculator {
    /// Calculate PDN.
    /// Formula: Total monthly payments / Average monthly income
    pub fn calculate(
        existing_obligations: &[Obligation],
        income: &IncomeSource,
        new_payment: Decimal,
        _loan_amount: Decimal,
    ) -> Result<PdnResult, &'static str> {
        // 6960-U / 7226-U: consumer loans require confirmed income from 01.01.2026.
        if !income.is_confirmed() {
            return Err("Unconfirmed income (Declared/Estimated) is not accepted per 6960-U / 7226-U; use Confirmed or FamilyTotal");
        }

        let total_existing: Decimal = existing_obligations
            .iter()
            .map(|o| o.recognized_payment())
            .sum();
        let total_payments = total_existing + new_payment;

        let inc_val = income.value();

        let raw_ratio = safe_div(total_payments, inc_val);
        // Financial rounding to 2 decimal places per standard
        let ratio = round_financial(raw_ratio);

        Ok(PdnResult {
            ratio,
            total_payments,
            is_risky: ratio > Decimal::new(50, 2), // > 0.50
        })
    }
}

// ---- TESTS -------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(not(feature = "std"))]
    use alloc::vec;
    #[cfg(feature = "std")]
    use std::vec;

    #[test]
    fn calculate_pdn_standard_loan() {
        let existing = vec![Obligation {
            monthly_payment: Decimal::new(15_000, 0),
            is_credit_card: false,
            credit_limit: None,
        }];
        let income = IncomeSource::Confirmed(Decimal::new(60_000, 0));
        let new_payment = Decimal::new(10_000, 0);
        let loan_amount = Decimal::new(100_000, 0);

        let res = PdnCalculator::calculate(&existing, &income, new_payment, loan_amount).unwrap();
        assert_eq!(res.total_payments, Decimal::new(25_000, 0));
        assert_eq!(res.ratio, Decimal::new(42, 2)); // 25000/60000 = 0.4166.. -> 0.42
        assert!(!res.is_risky);
    }

    #[test]
    fn calculate_pdn_credit_card_5_percent_limit() {
        // Payment is 1000, but limit is 100,000. 5% of 100k is 5,000.
        // It should use 5,000.
        let existing = vec![Obligation {
            monthly_payment: Decimal::new(1_000, 0),
            is_credit_card: true,
            credit_limit: Some(Decimal::new(100_000, 0)),
        }];
        let income = IncomeSource::Confirmed(Decimal::new(30_000, 0));
        let new_payment = Decimal::new(10_000, 0); // Total: 15,000
        let loan_amount = Decimal::new(80_000, 0);

        let res = PdnCalculator::calculate(&existing, &income, new_payment, loan_amount).unwrap();
        assert_eq!(res.total_payments, Decimal::new(15_000, 0));
        assert_eq!(res.ratio, Decimal::new(50, 2));
        assert!(!res.is_risky); // <= 0.50 is not risky, > 0.50 is
    }

    #[test]
    fn pdn_zero_income_confirmed_safe() {
        let existing = vec![];
        let income = IncomeSource::Confirmed(Decimal::ZERO);
        let new_payment = Decimal::new(10_000, 0);
        let loan_amount = Decimal::new(100_000, 0);

        let res = PdnCalculator::calculate(&existing, &income, new_payment, loan_amount).unwrap();
        assert_eq!(res.ratio, Decimal::ZERO); // safe_div returns Decimal::ZERO
    }

    #[test]
    fn pdn_rejects_declared_income_regardless_of_amount() {
        let existing = vec![];
        let income = IncomeSource::Declared(Decimal::new(40_000, 0));
        let new_payment = Decimal::new(5_000, 0);
        let loan_amount = Decimal::new(300_000, 0);

        // Fails because income is unconfirmed regardless of loan amount.
        let err =
            PdnCalculator::calculate(&existing, &income, new_payment, loan_amount).unwrap_err();
        assert!(err.contains("6960-U"));

        let income_official = IncomeSource::Confirmed(Decimal::new(40_000, 0));
        assert!(
            PdnCalculator::calculate(&existing, &income_official, new_payment, loan_amount).is_ok()
        );
    }
}

#[cfg(test)]
mod proptests {
    use super::*;
    use proptest::prelude::*;
    use rfe_types::rust_decimal::prelude::FromPrimitive;

    proptest! {
        #[test]
        fn test_pdn_invariants(
            income_val in 1000..1_000_000i64,
            new_payment_val in 0..100_000i64,
            existing_payment_val in 0..100_000i64,
        ) {
            let income = IncomeSource::Confirmed(Decimal::from_i64(income_val).unwrap());
            let new_payment = Decimal::from_i64(new_payment_val).unwrap();
            let existing = alloc::vec![Obligation {
                monthly_payment: Decimal::from_i64(existing_payment_val).unwrap(),
                is_credit_card: false,
                credit_limit: None,
            }];

            let res = PdnCalculator::calculate(&existing, &income, new_payment, Decimal::new(100_000, 0)).unwrap();

            // Invariant: total payments must be sum of existing and new
            prop_assert_eq!(res.total_payments, new_payment + Decimal::from_i64(existing_payment_val).unwrap());

            // Invariant: ratio should be proportional to payments
            if res.total_payments > Decimal::ZERO {
                 prop_assert!(res.ratio >= Decimal::ZERO);
            }
        }
    }
}
