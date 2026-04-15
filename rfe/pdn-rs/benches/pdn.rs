use criterion::{black_box, criterion_group, criterion_main, Criterion};
use pdn_rs::{IncomeSource, Obligation, PdnCalculator};
use rfe_types::rust_decimal::Decimal;

fn bench_pdn_calculation(c: &mut Criterion) {
    let obligations: Vec<Obligation> = (0..50)
        .map(|i| Obligation {
            monthly_payment: Decimal::new(1000 + i as i64 * 10, 0),
            is_credit_card: i % 2 == 0,
            credit_limit: if i % 2 == 0 {
                Some(Decimal::new(50000, 0))
            } else {
                None
            },
        })
        .collect();

    let income = IncomeSource::Confirmed(Decimal::new(150_000, 0));
    let new_payment = Decimal::new(5_000, 0);

    c.bench_function("pdn_calculate_50_obligations_6960u", |b| {
        b.iter(|| {
            PdnCalculator::calculate(
                black_box(&obligations),
                black_box(&income),
                black_box(new_payment),
                black_box(Decimal::new(100_000, 0)),
            )
        })
    });
}

criterion_group!(benches, bench_pdn_calculation);
criterion_main!(benches);
