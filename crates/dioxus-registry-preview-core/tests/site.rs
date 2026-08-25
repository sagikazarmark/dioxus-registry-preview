use std::path::PathBuf;

use dioxus_registry_preview_core::{MarkdownOptions, load_site_from_catalog};

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn loads_a_valid_documentation_site_from_disk() {
    let validation =
        load_site_from_catalog(&fixture("valid/catalog.rs"), &MarkdownOptions::default());

    assert_eq!(validation.diagnostics, []);
    let site = validation.value.expect("the fixture should produce a site");
    assert_eq!(site.default_component_module, "widget");
    assert_eq!(
        site.groups
            .iter()
            .map(|group| group.id.as_str())
            .collect::<Vec<_>>(),
        ["shared", "fixtures"]
    );
    assert_eq!(site.components[0].name, "widget");
    assert_eq!(site.components[0].title, "Widget");
    assert_eq!(site.components[0].group_id, "fixtures");
    assert_eq!(site.components[0].examples[0].slug, "overview");
    assert_eq!(site.pages[0].id, "widget");
    assert!(site.components[0].readme_html.contains("<table>"));
    assert!(
        site.components[0]
            .readme_html
            .contains("<del>available</del>")
    );
}

#[test]
fn finds_the_catalog_invocation_inside_an_inline_module() {
    let validation = load_site_from_catalog(
        &fixture("valid/nested-catalog.rs"),
        &MarkdownOptions::default(),
    );

    assert_eq!(validation.diagnostics, []);
    assert_eq!(
        validation
            .value
            .expect("the valid nested catalog should produce a site")
            .default_component_module,
        "widget"
    );
}
