#[test]
fn typestate_compile_fail_suite() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/compile_fail/*.rs");
}
