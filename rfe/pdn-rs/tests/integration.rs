use pdn_rs::{IncomeSource, Obligation, PdnCalculator};
use proptest::prelude::*;
use rfe_types::rust_decimal::Decimal;

#[test]
fn credit_card_5pct_rule_6960u_regression_vector() {
    let obligation = Obligation {
        monthly_payment: Decimal::ZERO,
        is_credit_card: true,
        credit_limit: Some(Decimal::new(100_000, 0)),
    };

    assert_eq!(obligation.recognized_payment(), Decimal::new(5_000, 0));
}

proptest! {
    #[test]
    fn doesnt_crash_on_random_values(
        payment in 0u64..1_000_000,
        income in 0u64..1_000_000,
        new_pay in 0u64..1_000_000
    ) {
        let existing = vec![Obligation {
            monthly_payment: Decimal::from(payment),
            is_credit_card: false,
            credit_limit: None,
        }];
        let inc = IncomeSource::Confirmed(Decimal::from(income));
        assert!(PdnCalculator::calculate(&existing, &inc, Decimal::from(new_pay), Decimal::from(60_000)).is_ok());
    }
}

#[test]
fn credit_card_uses_actual_payment_when_higher_than_5pct() {
    let obligation = Obligation {
        monthly_payment: Decimal::new(8_000, 0),
        is_credit_card: true,
        credit_limit: Some(Decimal::new(100_000, 0)),
    };
    assert_eq!(obligation.recognized_payment(), Decimal::new(8_000, 0));
}
