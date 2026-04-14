use criterion::{black_box, criterion_group, criterion_main, Criterion};

// Benchmark: BLAKE3 vs Streebog-256 on a 32-byte chain root re-hash.
// BLAKE3 is used for internal audit chaining (fast, no_std).
// Streebog-256 is used exclusively at the export boundary (CBR / SMEV 4).
// This bench quantifies the latency penalty of regulatory export.

fn bench_streebog_vs_blake3(c: &mut Criterion) {
    let input = [0xabu8; 32]; // non-zero representative chain root

    c.bench_function("blake3_32b", |b| b.iter(|| blake3::hash(black_box(&input))));

    c.bench_function("streebog256_32b", |b| {
        b.iter(|| {
            use streebog::{Digest, Streebog256};
            Streebog256::digest(black_box(&input))
        })
    });

    c.bench_function("audit_root_gost_export_32b", |b| {
        b.iter(|| rfe_types::audit_root_gost_export(black_box(&input)))
    });
}

criterion_group!(benches, bench_streebog_vs_blake3);
criterion_main!(benches);
