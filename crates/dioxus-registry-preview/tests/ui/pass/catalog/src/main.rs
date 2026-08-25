use dioxus::prelude::*;

mod example {
    use dioxus::prelude::*;
    use dioxus_registry_preview::ExampleDocumentation;

    #[component]
    pub fn ExampleSection(documentation: ExampleDocumentation, children: Element) -> Element {
        rsx! {
            section { "data-example": documentation.slug, {children} }
        }
    }
}

#[derive(Copy, Clone, Debug)]
enum Group {
    Fixtures,
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum PageKind {
    Component(ComponentPage),
    Rsx,
}

type PageDescriptor = dioxus_registry_preview::PageDescriptor<Group, PageKind>;

const ABOUT: PageDescriptor = PageDescriptor::new(
    "about",
    "/about",
    "About",
    "About this fixture.",
    dioxus_registry_preview::NavigationPlacement::Standalone("Fixture"),
    dioxus_registry_preview::ListingPolicy::Listed,
    dioxus_registry_preview::BrowserTestPolicy::Disabled,
    PageKind::Rsx,
);

dioxus_registry_preview::component_pages! {
    manifest: "../registry.json",
    repository: "https://github.com/acme/catalog-fixture",
    revision: "v1.2.3",
    group: Group {
        "shared" => Group::Fixtures,
        "fixtures" => Group::Fixtures,
    },
    default: widget,
    catalog: PageDescriptor {
        path: "/components",
        component: PageKind::Component,
        custom: [ABOUT],
    },
}

fn main() {
    assert_eq!(APP_CATALOG.description(), "A generated catalog fixture.");
    let registry = APP_CATALOG
        .registry()
        .expect("the fixture configures Registry installation facts");
    assert_eq!(registry.name(), "catalog-fixture");
    assert_eq!(registry.components(), ["widget"]);
    assert_eq!(registry.default_component(), "widget");
    assert_eq!(registry.source().repository(), "https://github.com/acme/catalog-fixture");
    assert_eq!(registry.source().revision(), Some("v1.2.3"));
    assert_eq!(APP_CATALOG.group_id(&PAGE_CATALOG[1]), "fixtures");
    assert!(matches!(ComponentPage::default().group(), Group::Fixtures));
    let _: Element = ComponentPage::default().view();
    assert_eq!(PAGE_CATALOG[0].id, ABOUT.id);
    assert_eq!(PAGE_CATALOG[1].id, "widget");
    assert_eq!(PAGE_CATALOG[1].path, "/components/widget");
    assert_eq!(
        PAGE_CATALOG[1].browser_tests,
        dioxus_registry_preview::BrowserTestPolicy::Enabled
    );
}
