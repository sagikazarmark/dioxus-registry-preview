#[test]
fn component_macro_generates_composable_default_page() {
    let tests = trybuild::TestCases::new();

    tests.pass("tests/ui/pass/default-page/docs/mod.rs");
}

#[test]
fn component_pages_resolves_stable_group_ids() {
    let tests = trybuild::TestCases::new();

    tests.pass("tests/ui/pass/catalog/src/main.rs");
}

#[test]
fn component_macros_report_authoring_errors() {
    let tests = trybuild::TestCases::new();

    tests.compile_fail("tests/ui/fail/duplicate-group.rs");
    tests.compile_fail("tests/ui/fail/duplicate-slug.rs");
    tests.compile_fail("tests/ui/fail/missing-example/docs/mod.rs");
    tests.compile_fail("tests/ui/fail/bad-readme/docs/mod.rs");
    tests.compile_fail("tests/ui/fail/unknown-group/src/main.rs");
    tests.compile_fail("tests/ui/fail/invalid-repository.rs");
    tests.compile_fail("tests/ui/fail/missing-repository.rs");
    tests.compile_fail("tests/ui/fail/invalid-revision.rs");
    tests.compile_fail("tests/ui/fail/invalid-registry-name.rs");
    tests.compile_fail("tests/ui/fail/raw-example.rs");
}
