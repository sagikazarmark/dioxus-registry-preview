use std::path::PathBuf;

use dioxus_registry_preview::validation::{
    DiagnosticCode, MarkdownHooks, MarkdownOptions, load_site_from_catalog,
    validate_registry_source,
};

struct ConsumerMarkdownHooks;

impl MarkdownHooks for ConsumerMarkdownHooks {
    fn code_block(&self, _language: Option<&str>, source: &str) -> Option<String> {
        Some(format!("<pre>{source}</pre>"))
    }
}

#[test]
fn facade_validation_reports_every_independent_site_error() {
    let catalog = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../dioxus-registry-preview-core/tests/fixtures/broken/catalog.rs");
    let hooks = ConsumerMarkdownHooks;
    let options = MarkdownOptions::default().with_hooks(&hooks);
    let diagnostics = load_site_from_catalog(&catalog, &options).diagnostics;
    let codes: Vec<_> = diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect();

    for expected in [
        DiagnosticCode::DuplicateMemberModule,
        DiagnosticCode::InvalidComponentName,
        DiagnosticCode::MissingPublicComponent,
        DiagnosticCode::MalformedReadme,
        DiagnosticCode::DuplicateExampleSlug,
        DiagnosticCode::UnknownGroup,
        DiagnosticCode::MissingExclude,
        DiagnosticCode::InvalidPage,
    ] {
        assert!(
            codes.contains(&expected),
            "missing {expected:?} in {codes:?}"
        );
    }
    assert!(diagnostics.len() > 10, "diagnostics were not aggregated");
}

#[test]
fn facade_exports_registry_source_validation() {
    let source = validate_registry_source(Some("https://github.com/acme/components"), None);

    assert_eq!(source.diagnostics, []);
    assert_eq!(
        source
            .value
            .expect("a valid Registry source should produce a value")
            .repository,
        "https://github.com/acme/components"
    );
}
