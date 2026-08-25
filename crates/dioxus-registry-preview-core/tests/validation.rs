use std::fs;
use std::path::{Path, PathBuf};

use dioxus_registry_preview_core::{
    CatalogInvocation, ComponentInvocation, DiagnosticCode, MarkdownHooks, MarkdownOptions,
    component_invocation, example_section_name, load_catalog, load_component_with_markdown,
    load_site_from_catalog, markdown_to_html, readme_parts, sentence_case,
    validate_registry_source,
};
use syn::parse_str;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

fn codes(path: &Path) -> Vec<DiagnosticCode> {
    load_site_from_catalog(path, &MarkdownOptions::default())
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect()
}

#[test]
fn reports_independent_site_errors_in_one_validation() {
    let codes = codes(&fixture("broken/catalog.rs"));

    for expected in [
        DiagnosticCode::DuplicateMemberModule,
        DiagnosticCode::InvalidComponentName,
        DiagnosticCode::MissingPublicComponent,
        DiagnosticCode::MalformedReadme,
        DiagnosticCode::DuplicateExampleSlug,
        DiagnosticCode::ExampleRead,
        DiagnosticCode::UnknownGroup,
        DiagnosticCode::MissingExclude,
        DiagnosticCode::SupersededLayout,
        DiagnosticCode::MissingDocsModule,
        DiagnosticCode::MissingExamplesDirectory,
        DiagnosticCode::InvalidPage,
        DiagnosticCode::InvalidDefault,
    ] {
        assert!(
            codes.contains(&expected),
            "missing {expected:?} in {codes:?}"
        );
    }
    assert!(
        codes.len() > 10,
        "diagnostics were not aggregated: {codes:?}"
    );
}

#[test]
fn reports_catalog_source_and_registry_manifest_failures_with_stable_codes() {
    let mappings = [
        (DiagnosticCode::UnsupportedSchema, "RDOC001"),
        (DiagnosticCode::ConfigRead, "RDOC002"),
        (DiagnosticCode::ConfigParse, "RDOC003"),
        (DiagnosticCode::RegistryManifestRead, "RDOC004"),
        (DiagnosticCode::RegistryManifestParse, "RDOC005"),
        (DiagnosticCode::EmptyRegistry, "RDOC006"),
        (DiagnosticCode::InvalidMemberPath, "RDOC007"),
        (DiagnosticCode::DuplicateMemberModule, "RDOC008"),
        (DiagnosticCode::ComponentManifestRead, "RDOC009"),
        (DiagnosticCode::ComponentManifestParse, "RDOC010"),
        (DiagnosticCode::InvalidComponentName, "RDOC011"),
        (DiagnosticCode::ComponentSourceRead, "RDOC012"),
        (DiagnosticCode::ComponentSourceParse, "RDOC013"),
        (DiagnosticCode::MissingPublicComponent, "RDOC014"),
        (DiagnosticCode::ReadmeRead, "RDOC015"),
        (DiagnosticCode::MalformedReadme, "RDOC016"),
        (DiagnosticCode::ExampleRead, "RDOC017"),
        (DiagnosticCode::InvalidGroupId, "RDOC018"),
        (DiagnosticCode::DuplicateGroupId, "RDOC019"),
        (DiagnosticCode::EmptyExamples, "RDOC020"),
        (DiagnosticCode::EmptyExampleSlug, "RDOC021"),
        (DiagnosticCode::DuplicateExampleSlug, "RDOC022"),
        (DiagnosticCode::InvocationRead, "RDOC023"),
        (DiagnosticCode::InvocationParse, "RDOC024"),
        (DiagnosticCode::MissingInvocation, "RDOC025"),
        (DiagnosticCode::UnknownGroup, "RDOC026"),
        (DiagnosticCode::MissingExclude, "RDOC027"),
        (DiagnosticCode::SupersededLayout, "RDOC028"),
        (DiagnosticCode::MissingDocsModule, "RDOC029"),
        (DiagnosticCode::MissingExamplesDirectory, "RDOC030"),
        (DiagnosticCode::InvalidPage, "RDOC031"),
        (DiagnosticCode::InvalidDefault, "RDOC032"),
        (DiagnosticCode::DuplicatePageId, "RDOC033"),
        (DiagnosticCode::MissingRequiredSection, "RDOC034"),
        (DiagnosticCode::SectionPatternMismatch, "RDOC035"),
        (DiagnosticCode::EmptyGroupMapping, "RDOC036"),
        (DiagnosticCode::MissingRegistryRepository, "RDOC037"),
        (DiagnosticCode::InvalidRegistryRepository, "RDOC038"),
        (DiagnosticCode::InvalidRegistryRevision, "RDOC039"),
        (DiagnosticCode::InvalidRegistryName, "RDOC040"),
    ];

    for (code, expected) in mappings {
        assert_eq!(code.as_str(), expected);
    }
    assert_eq!(
        codes(&fixture("does-not-exist.rs")),
        [DiagnosticCode::InvocationRead]
    );
    assert_eq!(
        codes(&fixture("malformed-catalog.rs")),
        [DiagnosticCode::InvocationParse]
    );
    assert_eq!(
        codes(&fixture("no-catalog.rs")),
        [DiagnosticCode::MissingInvocation]
    );
    assert_eq!(
        codes(&fixture("missing-registry-catalog.rs")),
        [DiagnosticCode::RegistryManifestRead]
    );
    assert_eq!(
        codes(&fixture("malformed-registry-catalog.rs")),
        [DiagnosticCode::RegistryManifestParse]
    );

    let invalid_groups = codes(&fixture("invalid-groups-catalog.rs"));
    assert!(invalid_groups.contains(&DiagnosticCode::InvalidGroupId));
    assert!(invalid_groups.contains(&DiagnosticCode::DuplicateGroupId));
    assert!(invalid_groups.contains(&DiagnosticCode::UnknownGroup));
}

#[test]
fn component_rules_are_checked_through_the_loading_boundary() {
    let docs_path = fixture("valid/components/widget/docs/mod.rs");
    let invalid: ComponentInvocation = parse_str(
        r#"
            group: "Not Valid",
            examples: {
                first { slug: "", description: "Empty." },
                second { slug: "same", description: "First." },
                third { slug: "same", description: "Second." },
            },
        "#,
    )
    .expect("the intentionally invalid Component invocation should parse");
    let result = load_component_with_markdown(&docs_path, &invalid, &MarkdownOptions::default());
    let codes = result
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();

    assert!(codes.contains(&DiagnosticCode::InvalidGroupId));
    assert!(codes.contains(&DiagnosticCode::EmptyExampleSlug));
    assert!(codes.contains(&DiagnosticCode::DuplicateExampleSlug));

    let empty: ComponentInvocation = parse_str(r#"group: "fixtures", examples: {}"#)
        .expect("the empty-Examples invocation should parse");
    let result = load_component_with_markdown(&docs_path, &empty, &MarkdownOptions::default());
    assert!(
        result
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::EmptyExamples)
    );
}

#[test]
fn component_loading_reports_malformed_and_missing_authoring_files() {
    let input: ComponentInvocation = parse_str(
        r#"
            group: "fixtures",
            examples: {
                overview { description: "An example." },
            },
        "#,
    )
    .expect("the valid Component invocation should parse");
    let malformed = load_component_with_markdown(
        &fixture("component-malformed/docs/mod.rs"),
        &input,
        &MarkdownOptions::default(),
    );
    let malformed_codes = malformed
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    assert!(malformed_codes.contains(&DiagnosticCode::ComponentManifestParse));
    assert!(malformed_codes.contains(&DiagnosticCode::ComponentSourceParse));
    assert!(malformed_codes.contains(&DiagnosticCode::MalformedReadme));

    let missing = load_component_with_markdown(
        &fixture("missing/docs/mod.rs"),
        &input,
        &MarkdownOptions::default(),
    );
    let missing_codes = missing
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    assert!(missing_codes.contains(&DiagnosticCode::ComponentManifestRead));
    assert!(missing_codes.contains(&DiagnosticCode::ComponentSourceRead));
    assert!(missing_codes.contains(&DiagnosticCode::ReadmeRead));
    assert!(missing_codes.contains(&DiagnosticCode::ExampleRead));

    let absolute: ComponentInvocation = parse_str(&format!(
        r#"component_root: {:?}, group: "fixtures", examples: {{ overview {{ description: "An example." }} }}"#,
        fixture("valid/components/widget").display().to_string()
    ))
    .expect("the absolute-root Component invocation should parse");
    let result = load_component_with_markdown(
        &fixture("valid/components/widget/docs/mod.rs"),
        &absolute,
        &MarkdownOptions::default(),
    );
    assert_eq!(
        result.diagnostics[0].code,
        DiagnosticCode::InvalidMemberPath
    );
}

#[test]
fn catalog_rules_are_checked_without_macro_expansion() {
    let invocation_path = fixture("valid/catalog.rs");
    let input: CatalogInvocation = parse_str(
        r#"
            manifest: "component.json",
            group: ComponentGroup {
                "fixtures" => ComponentGroup::Fixtures,
            },
            default: widget,
            catalog: () { path: "/components", component: PageKind::Component, custom: [] },
        "#,
    )
    .expect("the valid catalog invocation should parse");
    let catalog = load_catalog(&invocation_path, &input);
    assert_eq!(catalog.diagnostics, []);
    let catalog = catalog
        .value
        .expect("catalog loading should produce a value without diagnostics");
    assert_eq!(catalog.registry_name, "fixture-registry");
    assert_eq!(
        catalog.registry_description,
        "A valid documentation Registry fixture."
    );
    assert_eq!(catalog.default_index, 0);
    assert_eq!(catalog.default_component, "widget");
    assert_eq!(catalog.entries[0].module, "widget");
    assert_eq!(catalog.entries[0].group_id, "fixtures");
    assert_eq!(catalog.entries[0].id, "widget");
    assert_eq!(catalog.entries[0].title, "Widget");
    assert!(catalog.entries[0].listed);

    let invalid: CatalogInvocation = parse_str(
        r#"
            manifest: "component.json",
            group: ComponentGroup {
                "Bad" => ComponentGroup::Fixtures,
                "Bad" => ComponentGroup::Fixtures,
            },
            default: absent,
            catalog: () { path: "/components", component: PageKind::Component, custom: [] },
        "#,
    )
    .expect("the invalid catalog values should still parse");
    let codes = load_catalog(&invocation_path, &invalid)
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    assert!(codes.contains(&DiagnosticCode::InvalidGroupId));
    assert!(codes.contains(&DiagnosticCode::DuplicateGroupId));
    assert!(codes.contains(&DiagnosticCode::UnknownGroup));
    assert!(codes.contains(&DiagnosticCode::InvalidDefault));

    let no_groups: CatalogInvocation = parse_str(
        r#"
            manifest: "component.json",
            group: ComponentGroup {},
            default: widget,
            catalog: () { path: "/components", component: PageKind::Component, custom: [] },
        "#,
    )
    .expect("the empty-group catalog invocation should parse");
    assert!(
        load_catalog(&invocation_path, &no_groups)
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.code == DiagnosticCode::EmptyGroupMapping)
    );
}

#[test]
fn registry_source_validation_accepts_git_urls_and_reports_stable_diagnostics() {
    let source =
        validate_registry_source(Some("https://github.com/acme/components"), Some("v1.2.3"));

    assert_eq!(source.diagnostics, []);
    let source = source
        .value
        .expect("a valid Registry source should produce a value");
    assert_eq!(source.repository, "https://github.com/acme/components");
    assert_eq!(source.revision.as_deref(), Some("v1.2.3"));

    let missing = validate_registry_source(None, None);
    assert_eq!(missing.diagnostics[0].code.as_str(), "RDOC037");

    let invalid = validate_registry_source(Some("../components"), None);
    assert_eq!(invalid.diagnostics[0].code.as_str(), "RDOC038");

    let invalid_revision =
        validate_registry_source(Some("ssh://git@github.com/acme/components"), Some(""));
    assert_eq!(invalid_revision.diagnostics[0].code.as_str(), "RDOC039");

    let unicode_whitespace = validate_registry_source(
        Some("https://github.com/acme/components"),
        Some("release\u{2003}candidate"),
    );
    assert_eq!(unicode_whitespace.diagnostics[0].code.as_str(), "RDOC039");
}

#[test]
fn catalog_loading_reports_manifest_and_member_failures_directly() {
    let invocation_path = fixture("catalog-errors/catalog.rs");
    let missing: CatalogInvocation = parse_str(
        r#"
            manifest: "missing.json",
            group: ComponentGroup { "fixtures" => ComponentGroup::Fixtures },
            default: widget,
            catalog: () { path: "/components", component: PageKind::Component, custom: [] },
        "#,
    )
    .expect("the missing-manifest catalog invocation should parse");
    assert_eq!(
        load_catalog(&invocation_path, &missing).diagnostics[0].code,
        DiagnosticCode::RegistryManifestRead
    );

    let malformed: CatalogInvocation = parse_str(
        r#"
            manifest: "malformed.json",
            group: ComponentGroup { "fixtures" => ComponentGroup::Fixtures },
            default: widget,
            catalog: () { path: "/components", component: PageKind::Component, custom: [] },
        "#,
    )
    .expect("the malformed-manifest catalog invocation should parse");
    assert_eq!(
        load_catalog(&invocation_path, &malformed).diagnostics[0].code,
        DiagnosticCode::RegistryManifestParse
    );

    let empty: CatalogInvocation = parse_str(
        r#"
            manifest: "empty.json",
            group: ComponentGroup { "fixtures" => ComponentGroup::Fixtures },
            default: widget,
            catalog: () { path: "/components", component: PageKind::Component, custom: [] },
        "#,
    )
    .expect("the empty-manifest catalog invocation should parse");
    let empty_codes = load_catalog(&invocation_path, &empty)
        .diagnostics
        .into_iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    assert!(empty_codes.contains(&DiagnosticCode::EmptyRegistry));
    assert!(empty_codes.contains(&DiagnosticCode::InvalidRegistryName));
    assert!(empty_codes.contains(&DiagnosticCode::InvalidDefault));

    let members: CatalogInvocation = parse_str(
        r#"
            manifest: "members.json",
            group: ComponentGroup { "fixtures" => ComponentGroup::Fixtures },
            default: widget,
            catalog: () { path: "/components", component: PageKind::Component, custom: [] },
        "#,
    )
    .expect("the invalid-members catalog invocation should parse");
    let member_diagnostics = load_catalog(&invocation_path, &members).diagnostics;
    let member_codes = member_diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<Vec<_>>();
    assert!(member_codes.contains(&DiagnosticCode::InvalidMemberPath));
    assert!(member_codes.contains(&DiagnosticCode::DuplicateMemberModule));
    assert!(member_codes.contains(&DiagnosticCode::MissingInvocation));
    assert!(member_diagnostics.iter().any(|diagnostic| {
        diagnostic.message
            == "Registry member `components\\windows` must be a normalized relative path"
    }));
    assert!(member_diagnostics.iter().any(|diagnostic| {
        diagnostic.message
            == "Registry member `components/r#type` does not end in a Rust module name"
    }));

    let absolute: CatalogInvocation = parse_str(&format!(
        r#"
            manifest: {:?},
            group: ComponentGroup {{ "fixtures" => ComponentGroup::Fixtures }},
            default: widget,
            catalog: () {{ path: "/components", component: PageKind::Component, custom: [] }},
        "#,
        fixture("valid/component.json").display().to_string()
    ))
    .expect("the absolute-manifest catalog invocation should parse");
    assert_eq!(
        load_catalog(&invocation_path, &absolute).diagnostics[0].code,
        DiagnosticCode::InvalidMemberPath
    );
}

#[test]
fn invocation_parsers_reject_duplicate_unknown_and_missing_fields() {
    for source in [
        r#"group: "fixtures", group: "fixtures", examples: {}"#,
        r#"group: "fixtures", unknown: true, examples: {}"#,
        r#"group: "fixtures""#,
        r#"group: "fixtures", examples: { overview {} }"#,
        r#"group: "fixtures", examples: { r#type { description: "Raw." } }"#,
    ] {
        assert!(
            parse_str::<ComponentInvocation>(source).is_err(),
            "{source}"
        );
    }
    for source in [
        r#"manifest: "component.json", manifest: "other.json", group: Group {}, default: widget"#,
        r#"manifest: "component.json", repository: "https://example.com/one", repository: "https://example.com/two", group: Group {}, default: widget"#,
        r#"manifest: "component.json", revision: "v1", revision: "v2", group: Group {}, default: widget"#,
        r#"manifest: "component.json", unknown: Group {}, default: widget"#,
        r#"manifest: "component.json", group: Group {}"#,
    ] {
        assert!(parse_str::<CatalogInvocation>(source).is_err(), "{source}");
    }
}

#[test]
fn module_discovery_and_name_rules_are_public_core_behaviour() {
    let path = fixture("valid/components/widget/docs/mod.rs");
    let source = fs::read_to_string(&path).expect("the valid fixture source should be readable");
    let invocation = component_invocation(&source, &path)
        .expect("the valid fixture should contain a Component invocation");

    assert_eq!(invocation.group.value(), "fixtures");
    assert_eq!(sentence_case("floating_label"), "Floating label");
    assert_eq!(
        example_section_name("floating_label"),
        "FloatingLabelExampleSection"
    );
    assert_eq!(
        readme_parts("# Widget\n\nA useful widget.\n\n## Details\n\nBody."),
        Some(("Widget", "## Details\n\nBody."))
    );
    assert_eq!(
        readme_parts("# Widget\r\n\r\nA useful widget.\r\n\r\n## Details\r\n\r\nBody."),
        Some(("Widget", "## Details\r\n\r\nBody."))
    );
    assert_eq!(
        readme_parts("# Widget\n\nA useful widget."),
        Some(("Widget", ""))
    );
    assert_eq!(readme_parts("A useful widget."), None);
    assert_eq!(readme_parts("# \n\nA useful widget.\n\nBody."), None);
    assert_eq!(readme_parts("# Widget\n\n## Details\n\nBody."), None);

    let no_invocation = "fn helper() {}";
    assert_eq!(
        component_invocation(no_invocation, Path::new("docs/mod.rs"))
            .expect_err("the missing invocation should return a diagnostic")
            .code,
        DiagnosticCode::MissingInvocation
    );
    assert_eq!(
        component_invocation("fn", Path::new("docs/mod.rs"))
            .expect_err("malformed Rust should return a diagnostic")
            .code,
        DiagnosticCode::InvocationParse
    );
}

struct Hooks;

impl MarkdownHooks for Hooks {
    fn heading_anchor(
        &self,
        _level: pulldown_cmark::HeadingLevel,
        heading: &str,
    ) -> Option<String> {
        Some(heading.to_lowercase().replace(' ', "-"))
    }

    fn code_block(&self, language: Option<&str>, source: &str) -> Option<String> {
        Some(format!(
            "<pre data-language=\"{}\">{}</pre>",
            language.unwrap_or("text"),
            source.trim()
        ))
    }
}

#[test]
fn markdown_features_and_semantic_hooks_are_configurable() {
    let plain = markdown_to_html(
        "## Example heading\n\n~~old~~\n\n| A |\n| - |\n| B |",
        &MarkdownOptions::default()
            .with_strikethrough(false)
            .with_tables(false),
    );
    assert!(!plain.contains("<del>"));
    assert!(!plain.contains("<table>"));

    let hooked = markdown_to_html(
        "## Example heading\n\n```rust\nlet answer = 42;\n```",
        &MarkdownOptions::default().with_hooks(&Hooks),
    );
    assert!(hooked.contains("<h2 id=\"example-heading\">"));
    assert!(hooked.contains("<pre data-language=\"rust\">let answer = 42;</pre>"));
}
