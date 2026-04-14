use rfe_types::hashing::{hash_canonical_reports, hash_raw_request};
use rfe_types::{rust_decimal::Decimal, ComplianceReport, SealInput};

#[test]
fn test_seal_v1_determinism() {
    let nonce = [0xAAu8; 32];
    let chain_head_pre = [0x55u8; 32];

    // Request: Single line CSV
    let csv = b"TX001,100.00,500.00,1\n";
    let request_hash = hash_raw_request(csv);

    // Response: Single report
    let mut reports = vec![ComplianceReport {
        transaction_id: "TX001".to_string(),
        pdn_ratio: Decimal::new(2, 1), // 0.2
        is_pdn_risky: false,
        fraud_signs: vec![],
        recommendation: "APPROVE".to_string(),
        created_at_micros: 1712500000000,
    }];
    let result_hash = hash_canonical_reports(&mut reports).expect("canonical result hash");

    let seal_input = SealInput::new_v1(nonce, request_hash, result_hash, chain_head_pre);
    let seal_a = seal_input.compute_seal();

    // 1. Determinism check
    let seal_b = seal_input.compute_seal();
    assert_eq!(seal_a, seal_b, "Seal must be deterministic");

    // 2. Request perturbation (Dirty Bit Request)
    let csv_modified = b"TX001,100.01,500.00,1\n";
    let request_hash_mod = hash_raw_request(csv_modified);
    let seal_mod_req =
        SealInput::new_v1(nonce, request_hash_mod, result_hash, chain_head_pre).compute_seal();
    assert_ne!(seal_a, seal_mod_req, "Seal must reflect request changes");

    // 3. Result perturbation (Dirty Bit Result)
    let mut reports_mod = reports.clone();
    reports_mod[0].is_pdn_risky = true;
    let result_hash_mod = hash_canonical_reports(&mut reports_mod).expect("canonical result hash");
    let seal_mod_res =
        SealInput::new_v1(nonce, request_hash, result_hash_mod, chain_head_pre).compute_seal();
    assert_ne!(seal_a, seal_mod_res, "Seal must reflect result changes");

    // 4. Nonce perturbation
    let mut nonce_mod = nonce;
    nonce_mod[0] ^= 0xFF;
    let seal_mod_nonce =
        SealInput::new_v1(nonce_mod, request_hash, result_hash, chain_head_pre).compute_seal();
    assert_ne!(seal_a, seal_mod_nonce, "Seal must reflect nonce changes");
}

#[test]
fn test_canonical_sorting() {
    let mut reports = vec![
        ComplianceReport {
            transaction_id: "TX002".to_string(),
            pdn_ratio: Decimal::ZERO,
            is_pdn_risky: false,
            fraud_signs: vec![],
            recommendation: "A".to_string(),
            created_at_micros: 0,
        },
        ComplianceReport {
            transaction_id: "TX001".to_string(),
            pdn_ratio: Decimal::ZERO,
            is_pdn_risky: false,
            fraud_signs: vec![],
            recommendation: "B".to_string(),
            created_at_micros: 0,
        },
    ];

    let hash_a = hash_canonical_reports(&mut reports).expect("canonical result hash");

    // Shuffle and re-hash
    let mut reports_shuffled = vec![reports[1].clone(), reports[0].clone()];
    let hash_b = hash_canonical_reports(&mut reports_shuffled).expect("canonical result hash");

    assert_eq!(
        hash_a, hash_b,
        "Hash must be invariant to input insertion order due to canonical sorting"
    );
    assert_eq!(
        reports[0].transaction_id, "TX001",
        "Reports must be sorted in-place"
    );
}
