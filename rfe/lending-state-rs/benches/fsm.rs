use criterion::{black_box, criterion_group, criterion_main, Criterion};
use lending_state_rs::LoanApplication;
use rfe_types::{rust_decimal::Decimal, ClientId};

fn bench_fsm_transitions(c: &mut Criterion) {
    let client_id = ClientId::new();
    let amount = Decimal::new(10_000, 0);

    c.bench_function("10k_transitions", |b| {
        b.iter(|| {
            // We simulate a fast path straight to close to measure the hash chain overhead
            for _ in 0..10_000 {
                let draft = LoanApplication::new(black_box(client_id), black_box(amount));
                let scoring = draft.submit_to_scoring();
                let pdn = scoring.scoring_done(true).unwrap();
                let approved = pdn.pdn_done(false).unwrap();
                let disbursed = approved.disburse();
                let _closed = disbursed.close();
            }
        })
    });
}

criterion_group!(benches, bench_fsm_transitions);
criterion_main!(benches);
