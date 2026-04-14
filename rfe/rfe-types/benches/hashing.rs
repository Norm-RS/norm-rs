use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rfe_types::{blake3_hash, AuditEntry};

// Benchmarks: Blake3 single hash and audit chain step.
// These cover always-available hot paths (no feature gate).

fn bench_blake3_single(c: &mut Criterion) {
    let data = black_box([0u8; 1024]); // 1KB of data
    c.bench_function("blake3_hash_1kb", |b| b.iter(|| blake3_hash(&data)));
}

fn bench_audit_chain_step(c: &mut Criterion) {
    let payload = black_box([0u8; 256]);
    let genesis = AuditEntry::genesis(1712050000, &payload, None);

    c.bench_function("audit_entry_next", |b| {
        b.iter(|| genesis.next(1712050001, &payload, None))
    });
}

criterion_group!(benches, bench_blake3_single, bench_audit_chain_step);
criterion_main!(benches);
