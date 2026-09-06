use dioxus::prelude::*;
use dioxus_registry_preview::chrome::{
    App, AppCatalog, AppPage, AppProps, CatalogEntry, CodeBlock, ExampleSection, InstallationPage,
    PageCatalogManifest, PageCatalogManifestProps, RegistryDocumentation, RegistrySource, Shell,
    Sidebar, ThemeEntry, ThemeSwitcher, ThemeSwitcherManifest, ThemeSwitcherManifestProps,
};
use dioxus_registry_preview::{
    BrowserTestPolicy, ExampleDocumentation, ListingPolicy, NavigationPlacement, PageDescriptor,
};

#[derive(Clone, Copy, PartialEq)]
enum TestGroup {
    Guides,
}

fn test_group_id(page: &PageDescriptor<TestGroup, TestPage>) -> &'static str {
    match page.id {
        "installation" | "button" | "home" | "private" => "guides",
        _ => unreachable!(),
    }
}

#[derive(Clone, Copy, PartialEq)]
enum TestPage {
    Installation,
    Button,
    Home,
    Private,
}

impl AppPage for TestPage {
    fn kind(self) -> &'static str {
        "rsx"
    }

    fn render(self) -> Element {
        match self {
            Self::Installation => rsx! { h1 { "Install the Registry" } },
            Self::Button => rsx! { h1 { "Button reference" } },
            Self::Home => rsx! { h1 { "Consumer home ID" } },
            Self::Private => rsx! { h1 { "Private notes" } },
        }
    }
}

const APP_CATALOG: &[PageDescriptor<TestGroup, TestPage>] = &[PageDescriptor::new(
    "installation",
    "/",
    "Installation",
    "Install the Registry.",
    NavigationPlacement::Group(TestGroup::Guides),
    ListingPolicy::Listed,
    BrowserTestPolicy::Enabled,
    TestPage::Installation,
)];
const TEST_APP_CATALOG: AppCatalog<PageDescriptor<TestGroup, TestPage>> =
    AppCatalog::new(APP_CATALOG, test_group_id);

const TEST_REGISTRY: RegistryDocumentation = RegistryDocumentation::new(
    "example-registry",
    "A Registry used to test generated installation instructions.",
    &["button", "input"],
    "button",
    RegistrySource::git("https://github.com/acme/example-registry").with_revision("v1.2.3"),
);

const DEFAULT_HOME_PAGES: &[PageDescriptor<TestGroup, TestPage>] = &[
    PageDescriptor::new(
        "button",
        "/components/button",
        "Button",
        "Button component.",
        NavigationPlacement::Group(TestGroup::Guides),
        ListingPolicy::Listed,
        BrowserTestPolicy::Enabled,
        TestPage::Button,
    ),
    PageDescriptor::new(
        "home",
        "/home",
        "Authored home ID",
        "A Consumer page whose ID is home.",
        NavigationPlacement::Group(TestGroup::Guides),
        ListingPolicy::Listed,
        BrowserTestPolicy::Disabled,
        TestPage::Home,
    ),
    PageDescriptor::new(
        "private",
        "/private",
        "Private notes",
        "Unlisted Registry notes.",
        NavigationPlacement::Group(TestGroup::Guides),
        ListingPolicy::Unlisted,
        BrowserTestPolicy::Disabled,
        TestPage::Private,
    ),
];
const DEFAULT_HOME_CATALOG: AppCatalog<PageDescriptor<TestGroup, TestPage>> =
    AppCatalog::new(DEFAULT_HOME_PAGES, test_group_id)
        .with_description("A Registry used to test the default home.")
        .with_registry(TEST_REGISTRY);
const RESERVED_HOME_ID_PAGES: &[PageDescriptor<TestGroup, TestPage>] = &[PageDescriptor::new(
    "registry-home",
    "/consumer-home",
    "Reserved home ID",
    "A page that collides with the default home ID.",
    NavigationPlacement::Standalone("Fixture"),
    ListingPolicy::Listed,
    BrowserTestPolicy::Disabled,
    TestPage::Home,
)];
const RESERVED_HOME_ID_CATALOG: AppCatalog<PageDescriptor<TestGroup, TestPage>> =
    AppCatalog::new(RESERVED_HOME_ID_PAGES, test_group_id);
const RESERVED_INSTALLATION_ID_PAGES: &[PageDescriptor<TestGroup, TestPage>] =
    &[PageDescriptor::new(
        "registry-installation",
        "/consumer-installation",
        "Reserved installation ID",
        "A page that collides with the generated installation ID.",
        NavigationPlacement::Standalone("Fixture"),
        ListingPolicy::Listed,
        BrowserTestPolicy::Disabled,
        TestPage::Installation,
    )];
const RESERVED_INSTALLATION_ID_CATALOG: AppCatalog<PageDescriptor<TestGroup, TestPage>> =
    AppCatalog::new(RESERVED_INSTALLATION_ID_PAGES, test_group_id).with_registry(TEST_REGISTRY);
const AUTHORED_INSTALLATION_PAGES: &[PageDescriptor<TestGroup, TestPage>] = &[PageDescriptor::new(
    "consumer-installation",
    "/installation",
    "Consumer installation",
    "Consumer-owned installation instructions.",
    NavigationPlacement::Standalone("Getting started"),
    ListingPolicy::Listed,
    BrowserTestPolicy::Enabled,
    TestPage::Installation,
)];
const AUTHORED_INSTALLATION_CATALOG: AppCatalog<PageDescriptor<TestGroup, TestPage>> =
    AppCatalog::new(AUTHORED_INSTALLATION_PAGES, test_group_id).with_registry(TEST_REGISTRY);

fn catalog_entry() -> CatalogEntry {
    CatalogEntry::new(
        "installation",
        "/docs/installation",
        "Installation",
        "Install the Registry.",
        "rsx",
        "standalone",
        "Getting started",
        "listed",
        "disabled",
    )
}

fn sidebar_catalog() -> Vec<CatalogEntry> {
    vec![
        CatalogEntry::new(
            "button",
            "/components/button",
            "Button",
            "Button component.",
            "generated-component",
            "group",
            "actions",
            "listed",
            "enabled",
        ),
        CatalogEntry::new(
            "home",
            "/",
            "Home",
            "Registry home.",
            "rsx",
            "standalone",
            "Getting started",
            "unlisted",
            "disabled",
        ),
        catalog_entry(),
        CatalogEntry::new(
            "input",
            "/components/input",
            "Input",
            "Input component.",
            "generated-component",
            "group",
            "data-input",
            "listed",
            "enabled",
        ),
    ]
}

#[test]
fn page_catalog_manifest_writes_the_catalog_protocol() {
    let html = dioxus_ssr::render_element(rsx! {
        PageCatalogManifest {
            catalog: vec![catalog_entry()],
        }
    });

    assert_eq!(
        html,
        "<div data-page-catalog=\"true\" hidden=true><span data-catalog-page=\"installation\" data-path=\"/docs/installation\" data-title=\"Installation\" data-description=\"Install the Registry.\" data-kind=\"rsx\" data-navigation=\"standalone\" data-navigation-value=\"Getting started\" data-listing=\"listed\" data-browser-test=\"disabled\"></span></div>"
    );
}

#[test]
fn shell_mounts_the_default_theme_switcher() {
    let html = dioxus_ssr::render_element(rsx! {
        Shell {
            page_id: Some(String::from("installation")),
            catalog: vec![catalog_entry()],
            article { "Installation page" }
        }
    });
    let expected = "<div data-page-catalog=\"true\" hidden=true><span data-catalog-page=\"installation\" data-path=\"/docs/installation\" data-title=\"Installation\" data-description=\"Install the Registry.\" data-kind=\"rsx\" data-navigation=\"standalone\" data-navigation-value=\"Getting started\" data-listing=\"listed\" data-browser-test=\"disabled\"></span></div><main data-registry-preview-chrome=\"true\" data-page=\"installation\" class=\"rpv-shell\"><div data-switcher=\"theme\" hidden=true><input type=\"radio\" name=\"documentation-theme\" value=\"light\" checked=true data-value=\"light\" data-baseline=\"true\"/><input type=\"radio\" name=\"documentation-theme\" value=\"dark\" data-value=\"dark\" data-baseline=\"true\"/></div><div class=\"rpv-theme-switcher\" role=\"group\" aria-label=\"Color theme\"><span class=\"rpv-theme-switcher__label\">Theme</span><label class=\"rpv-theme-switcher__option\"><input type=\"radio\" name=\"documentation-theme-control\" value=\"light\" checked=true class=\"rpv-theme-switcher__control\"/><span class=\"rpv-theme-switcher__option-label\">Light</span></label><label class=\"rpv-theme-switcher__option\"><input type=\"radio\" name=\"documentation-theme-control\" value=\"dark\" class=\"rpv-theme-switcher__control\"/><span class=\"rpv-theme-switcher__option-label\">Dark</span></label></div><article>Installation page</article></main>";

    assert_eq!(html, expected);
}

#[test]
fn shell_preserves_an_explicit_consumer_theme_manifest() {
    let html = dioxus_ssr::render_element(rsx! {
        Shell {
            page_id: Some(String::from("installation")),
            catalog: vec![catalog_entry()],
            themes: vec![ThemeEntry::new("consumer-theme", true, true)],
            article { "Installation page" }
        }
    });
    let expected = "<div data-page-catalog=\"true\" hidden=true><span data-catalog-page=\"installation\" data-path=\"/docs/installation\" data-title=\"Installation\" data-description=\"Install the Registry.\" data-kind=\"rsx\" data-navigation=\"standalone\" data-navigation-value=\"Getting started\" data-listing=\"listed\" data-browser-test=\"disabled\"></span></div><div data-switcher=\"theme\" hidden=true><input type=\"radio\" name=\"documentation-theme\" value=\"consumer-theme\" checked=true data-value=\"consumer-theme\" data-baseline=\"true\"/></div><main data-registry-preview-chrome=\"true\" data-page=\"installation\" class=\"rpv-shell\"><article>Installation page</article></main>";

    assert_eq!(html, expected);
}

#[test]
fn theme_switcher_manifest_writes_the_theme_protocol() {
    let html = dioxus_ssr::render_element(rsx! {
        ThemeSwitcherManifest {
            themes: vec![
                ThemeEntry::new("light", true, true),
                ThemeEntry::new("dark", false, false),
            ],
        }
    });

    assert_eq!(
        html,
        "<div data-switcher=\"theme\" hidden=true><input type=\"radio\" name=\"documentation-theme\" value=\"light\" checked=true data-value=\"light\" data-baseline=\"true\"/><input type=\"radio\" name=\"documentation-theme\" value=\"dark\" data-value=\"dark\"/></div>"
    );
}

#[test]
#[should_panic(expected = "page catalog contains duplicate ID `installation`")]
fn page_catalog_manifest_rejects_duplicate_ids() {
    let _ = PageCatalogManifest(PageCatalogManifestProps {
        catalog: vec![catalog_entry(), catalog_entry()],
    });
}

#[test]
#[should_panic(expected = "page catalog IDs must be nonempty")]
fn page_catalog_manifest_rejects_empty_ids() {
    let mut entry = catalog_entry();
    entry.id = "";

    let _ = PageCatalogManifest(PageCatalogManifestProps {
        catalog: vec![entry],
    });
}

#[test]
#[should_panic(expected = "page catalog path `docs` must be absolute")]
fn page_catalog_manifest_rejects_relative_paths() {
    let mut entry = catalog_entry();
    entry.path = "docs";

    let _ = PageCatalogManifest(PageCatalogManifestProps {
        catalog: vec![entry],
    });
}

#[test]
#[should_panic(expected = "page catalog contains duplicate path `/docs/installation`")]
fn page_catalog_manifest_rejects_duplicate_paths() {
    let mut duplicate = catalog_entry();
    duplicate.id = "other";

    let _ = PageCatalogManifest(PageCatalogManifestProps {
        catalog: vec![catalog_entry(), duplicate],
    });
}

#[test]
#[should_panic(expected = "page `installation` has unsupported rendering kind `markdown`")]
fn page_catalog_manifest_rejects_unsupported_rendering_kinds() {
    let mut entry = catalog_entry();
    entry.kind = "markdown";

    let _ = PageCatalogManifest(PageCatalogManifestProps {
        catalog: vec![entry],
    });
}

#[test]
#[should_panic(expected = "page `installation` has unsupported navigation placement `hidden`")]
fn page_catalog_manifest_rejects_unsupported_navigation_placements() {
    let mut entry = catalog_entry();
    entry.navigation = "hidden";

    let _ = PageCatalogManifest(PageCatalogManifestProps {
        catalog: vec![entry],
    });
}

#[test]
#[should_panic(expected = "page `installation` must have a nonempty navigation value")]
fn page_catalog_manifest_rejects_empty_navigation_values() {
    let mut entry = catalog_entry();
    entry.navigation_value = "";

    let _ = PageCatalogManifest(PageCatalogManifestProps {
        catalog: vec![entry],
    });
}

#[test]
#[should_panic(expected = "page `installation` has unsupported listing policy `sometimes`")]
fn page_catalog_manifest_rejects_unsupported_listing_policies() {
    let mut entry = catalog_entry();
    entry.listing = "sometimes";

    let _ = PageCatalogManifest(PageCatalogManifestProps {
        catalog: vec![entry],
    });
}

#[test]
#[should_panic(expected = "page `installation` has unsupported browser-test policy `manual`")]
fn page_catalog_manifest_rejects_unsupported_browser_test_policies() {
    let mut entry = catalog_entry();
    entry.browser_test = "manual";

    let _ = PageCatalogManifest(PageCatalogManifestProps {
        catalog: vec![entry],
    });
}

#[test]
#[should_panic(expected = "theme manifest must not be empty")]
fn theme_switcher_manifest_requires_a_theme() {
    let _ = ThemeSwitcherManifest(ThemeSwitcherManifestProps { themes: vec![] });
}

#[test]
#[should_panic(expected = "theme IDs must be nonempty")]
fn theme_switcher_manifest_rejects_empty_ids() {
    let _ = ThemeSwitcherManifest(ThemeSwitcherManifestProps {
        themes: vec![ThemeEntry::new("", true, true)],
    });
}

#[test]
#[should_panic(expected = "theme manifest contains duplicate ID `light`")]
fn theme_switcher_manifest_rejects_duplicate_ids() {
    let _ = ThemeSwitcherManifest(ThemeSwitcherManifestProps {
        themes: vec![
            ThemeEntry::new("light", true, true),
            ThemeEntry::new("light", false, false),
        ],
    });
}

#[test]
#[should_panic(expected = "theme manifest must contain at least one baseline theme")]
fn theme_switcher_manifest_requires_a_baseline() {
    let _ = ThemeSwitcherManifest(ThemeSwitcherManifestProps {
        themes: vec![ThemeEntry::new("light", false, true)],
    });
}

#[test]
#[should_panic(expected = "theme manifest must contain exactly one checked theme")]
fn theme_switcher_manifest_requires_a_checked_theme() {
    let _ = ThemeSwitcherManifest(ThemeSwitcherManifestProps {
        themes: vec![ThemeEntry::new("light", true, false)],
    });
}

#[test]
#[should_panic(expected = "theme manifest must contain exactly one checked theme")]
fn theme_switcher_manifest_requires_one_checked_theme() {
    let _ = ThemeSwitcherManifest(ThemeSwitcherManifestProps {
        themes: vec![
            ThemeEntry::new("light", true, true),
            ThemeEntry::new("dark", true, true),
        ],
    });
}

#[test]
fn default_theme_switcher_exposes_visible_light_and_dark_choices() {
    let html = dioxus_ssr::render_element(rsx! { ThemeSwitcher {} });

    assert_eq!(
        html,
        "<div data-switcher=\"theme\" hidden=true><input type=\"radio\" name=\"documentation-theme\" value=\"light\" checked=true data-value=\"light\" data-baseline=\"true\"/><input type=\"radio\" name=\"documentation-theme\" value=\"dark\" data-value=\"dark\" data-baseline=\"true\"/></div><div class=\"rpv-theme-switcher\" role=\"group\" aria-label=\"Color theme\"><span class=\"rpv-theme-switcher__label\">Theme</span><label class=\"rpv-theme-switcher__option\"><input type=\"radio\" name=\"documentation-theme-control\" value=\"light\" checked=true class=\"rpv-theme-switcher__control\"/><span class=\"rpv-theme-switcher__option-label\">Light</span></label><label class=\"rpv-theme-switcher__option\"><input type=\"radio\" name=\"documentation-theme-control\" value=\"dark\" class=\"rpv-theme-switcher__control\"/><span class=\"rpv-theme-switcher__option-label\">Dark</span></label></div>"
    );
}

#[test]
fn installation_page_renders_registry_commands_configuration_and_members() {
    let html = dioxus_ssr::render_element(rsx! {
        InstallationPage { registry: TEST_REGISTRY }
    });

    assert!(html.contains("<h1 class=\"rpv-installation__title\">Install example-registry</h1>"));
    assert!(html.contains("A Registry used to test generated installation instructions."));
    assert!(
        html.contains("dx components add button --git &#39;https://github.com/acme/example-registry&#39; --rev &#39;v1.2.3&#39;"),
        "{html}"
    );
    assert!(html.contains("dx components add --all --git &#39;https://github.com/acme/example-registry&#39; --rev &#39;v1.2.3&#39;"));
    assert!(html.contains("registry = { git = &#34;https://github.com/acme/example-registry&#34;, rev = &#34;v1.2.3&#34; }"));
    let button = html
        .find("<code class=\"rpv-installation__component\">button</code>")
        .expect("the installation page should list button");
    let input = html
        .find("<code class=\"rpv-installation__component\">input</code>")
        .expect("the installation page should list input");
    assert!(button < input);
}

#[test]
fn installation_page_omits_revision_when_the_registry_is_unpinned() {
    let registry = RegistryDocumentation::new(
        "example-registry",
        "",
        &["button"],
        "button",
        RegistrySource::git("https://github.com/acme/example-registry"),
    );
    let html = dioxus_ssr::render_element(rsx! {
        InstallationPage { registry }
    });

    assert!(!html.contains("--rev"), "{html}");
    assert!(!html.contains("rev ="), "{html}");
}

#[test]
fn installation_page_escapes_shell_and_toml_source_values() {
    let registry = RegistryDocumentation::new(
        "example-registry",
        "",
        &["button"],
        "button",
        RegistrySource::git("https://example.com/acme's/\"registry\\name")
            .with_revision("release's/\"candidate\\one"),
    );
    let html = dioxus_ssr::render_element(rsx! {
        InstallationPage { registry }
    });

    assert!(
        html.contains(
            "&#39;https://example.com/acme&#39;&#34;&#39;&#34;&#39;s/&#34;registry\\name&#39;"
        ),
        "{html}"
    );
    assert!(
        html.contains("--rev &#39;release&#39;&#34;&#39;&#34;&#39;s/&#34;candidate\\one&#39;"),
        "{html}"
    );
    assert!(
        html.contains("git = &#34;https://example.com/acme&#39;s/\\&#34;registry\\\\name&#34;"),
        "{html}"
    );
    assert!(
        html.contains("rev = &#34;release&#39;s/\\&#34;candidate\\\\one&#34;"),
        "{html}"
    );
}

#[test]
fn sidebar_groups_listed_pages_and_marks_the_current_page() {
    fn app() -> Element {
        rsx! {
            Sidebar {
                page_id: Some(String::from("button")),
                catalog: sidebar_catalog(),
                href_for: |page: CatalogEntry| format!("{}?theme=dark", page.path),
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert_eq!(
        html,
        "<nav data-switcher=\"component\" class=\"rpv-sidebar\" aria-label=\"Documentation\"><section class=\"rpv-sidebar__section\"><h2 class=\"rpv-sidebar__heading\">actions</h2><ul class=\"rpv-sidebar__list\"><li><a data-value=\"button\" class=\"rpv-sidebar__link\" href=\"/components/button?theme=dark\" aria-current=\"page\">Button</a></li></ul></section><section class=\"rpv-sidebar__section\"><h2 class=\"rpv-sidebar__heading\">Getting started</h2><ul class=\"rpv-sidebar__list\"><li><a class=\"rpv-sidebar__link\" href=\"/docs/installation?theme=dark\">Installation</a></li></ul></section><section class=\"rpv-sidebar__section\"><h2 class=\"rpv-sidebar__heading\">data-input</h2><ul class=\"rpv-sidebar__list\"><li><a data-value=\"input\" class=\"rpv-sidebar__link\" href=\"/components/input?theme=dark\">Input</a></li></ul></section></nav>"
    );
}

#[test]
fn app_routes_catalog_pages_and_mounts_the_default_site_chrome() {
    fn app() -> Element {
        rsx! {
            App {
                catalog: TEST_APP_CATALOG,
                title: "Example Registry",
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("data-page=\"installation\""), "{html}");
    assert!(html.contains("data-catalog-page=\"installation\""));
    assert!(html.contains("data-navigation-value=\"guides\""));
    assert!(html.contains("data-kind=\"rsx\""));
    assert!(html.contains("<h1>Install the Registry</h1>"));
    assert!(html.contains("Example Registry"));
    assert!(html.contains("data-switcher=\"component\""));
    assert!(html.contains("data-switcher=\"theme\""));
    assert!(!html.contains("data-catalog-page=\"registry-home\""));
    assert!(html.contains("class=\"rpv-header__brand\" href=\"/\""));
}

#[test]
fn app_renders_a_default_home_when_the_catalog_does_not_claim_root() {
    fn app() -> Element {
        rsx! {
            App {
                catalog: DEFAULT_HOME_CATALOG,
                title: "Example Registry",
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(html.contains("data-page=\"registry-home\""), "{html}");
    assert!(html.contains("data-catalog-page=\"registry-installation\" data-path=\"/installation\" data-title=\"Installation\" data-description=\"Install components from this Registry.\" data-kind=\"rsx\" data-navigation=\"standalone\" data-navigation-value=\"Getting started\" data-listing=\"listed\" data-browser-test=\"enabled\""));
    assert!(html.contains("data-catalog-page=\"registry-home\" data-path=\"/\" data-title=\"Home\" data-description=\"A Registry used to test the default home.\" data-kind=\"rsx\" data-navigation=\"standalone\" data-navigation-value=\"Getting started\" data-listing=\"unlisted\" data-browser-test=\"enabled\""));
    assert!(html.contains("<h1 class=\"rpv-home__title\">Example Registry</h1>"));
    assert!(html.contains(
        "<p class=\"rpv-home__description\">A Registry used to test the default home.</p>"
    ));
    assert!(html.contains("class=\"rpv-home__cta\" href=\"/installation\">Getting started</a>"));
    assert!(html.contains("class=\"rpv-home__entry-title\">Installation</span><span class=\"rpv-home__entry-description\">Install components from this Registry.</span>"));
    assert!(html.contains("class=\"rpv-home__entry-title\">Button</span><span class=\"rpv-home__entry-description\">Button component.</span>"));
    assert!(html.contains("class=\"rpv-home__entry-title\">Authored home ID</span>"));
    assert!(!html.contains("class=\"rpv-home__entry-title\">Private notes</span>"));

    let standalone = html
        .find("<h2 class=\"rpv-home__section-title\">Getting started</h2>")
        .expect("the default home should include the standalone section");
    let group = html
        .find("<h2 class=\"rpv-home__section-title\">guides</h2>")
        .expect("the default home should include the Component group");
    assert!(standalone < group);
}

#[test]
fn an_authored_installation_path_suppresses_the_generated_page() {
    fn app() -> Element {
        rsx! {
            App {
                catalog: AUTHORED_INSTALLATION_CATALOG,
                title: "Example Registry",
            }
        }
    }

    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let html = dioxus_ssr::render(&dom);

    assert!(
        html.contains("data-catalog-page=\"consumer-installation\" data-path=\"/installation\"")
    );
    assert!(!html.contains("data-catalog-page=\"registry-installation\""));
    assert!(html.contains("class=\"rpv-home__cta\" href=\"/installation\""));
}

#[test]
#[should_panic(
    expected = "`registry-home` is reserved by chrome::App when it supplies the default home"
)]
fn app_rejects_the_reserved_home_id_when_synthesizing_root() {
    let _ = App(AppProps {
        catalog: RESERVED_HOME_ID_CATALOG,
        title: "Registry documentation",
    });
}

#[test]
#[should_panic(
    expected = "`registry-installation` is reserved by chrome::App when it supplies the installation page"
)]
fn app_rejects_the_reserved_installation_id_when_synthesizing_the_page() {
    let _ = App(AppProps {
        catalog: RESERVED_INSTALLATION_ID_CATALOG,
        title: "Registry documentation",
    });
}

const EXAMPLE_SOURCE: &str = "use dioxus::prelude::*;\n\n#[component]\npub fn Example() -> Element {\n    rsx! { button { \"Save\" } }\n}\n";

#[cfg(feature = "syntax-highlighting")]
const EXAMPLE: ExampleDocumentation = ExampleDocumentation::new(
    "overview",
    "Overview",
    "Shows the default button.",
    EXAMPLE_SOURCE,
    &[
        dioxus_registry_preview::code::HighlightSpan::new(0..3, "k"),
        dioxus_registry_preview::code::HighlightSpan::new(38..41, "k"),
        dioxus_registry_preview::code::HighlightSpan::new(42..44, "k"),
    ],
);

#[cfg(not(feature = "syntax-highlighting"))]
const EXAMPLE: ExampleDocumentation = ExampleDocumentation::new(
    "overview",
    "Overview",
    "Shows the default button.",
    EXAMPLE_SOURCE,
);

fn render_example_section() -> String {
    dioxus_ssr::render_element(rsx! {
        ExampleSection { documentation: EXAMPLE,
            button { "Save" }
        }
    })
}

#[test]
fn example_section_shows_the_preview_tab_and_keeps_the_code_panel_mounted() {
    let html = render_example_section();

    assert!(
        html.starts_with("<section data-example=\"overview\" class=\"rpv-example\">"),
        "{html}"
    );
    assert!(html.contains("<h2 class=\"rpv-example__title\">Overview</h2>"));
    assert!(html.contains("<p class=\"rpv-example__description\">Shows the default button.</p>"));
    assert!(html.contains(
        "<button type=\"button\" role=\"tab\" id=\"rpv-example-overview-preview-tab\" class=\"rpv-example__tab\" aria-selected=\"true\" aria-controls=\"rpv-example-overview-preview-panel\" tabindex=\"0\">Preview</button>"
    ), "{html}");
    assert!(html.contains(
        "<button type=\"button\" role=\"tab\" id=\"rpv-example-overview-code-tab\" class=\"rpv-example__tab\" aria-selected=\"false\" aria-controls=\"rpv-example-overview-code-panel\" tabindex=\"-1\">Code</button>"
    ), "{html}");
    assert!(html.contains(
        "<div id=\"rpv-example-overview-preview-panel\" role=\"tabpanel\" aria-labelledby=\"rpv-example-overview-preview-tab\" data-example-content=\"true\" class=\"rpv-example__content\"><button>Save</button></div>"
    ), "{html}");
    assert!(html.contains(
        "<div id=\"rpv-example-overview-code-panel\" role=\"tabpanel\" aria-labelledby=\"rpv-example-overview-code-tab\" hidden=true class=\"rpv-example__code\"><figure class=\"rpv-code-panel\">"
    ), "{html}");
    assert!(
        !html.contains("rpv-code-panel__label"),
        "the Code tab needs no caption: {html}"
    );
    assert!(html.contains(" Example() -&#62; Element {"), "{html}");
}

#[cfg(feature = "syntax-highlighting")]
#[test]
fn example_section_highlights_the_source_through_dioxus_code() {
    let html = render_example_section();

    assert!(
        html.contains("<pre class=\"dxc dxc-github-dark\" data-language=\"rust\"><code>"),
        "{html}"
    );
    assert!(html.contains("<span class=\"a-k\">use</span>"), "{html}");
    assert!(html.contains("<span class=\"a-k\">fn</span>"), "{html}");
    assert!(!html.contains("class=\"rpv-code\""), "{html}");
}

#[cfg(not(feature = "syntax-highlighting"))]
#[test]
fn example_section_falls_back_to_plain_source_without_highlighting() {
    let html = render_example_section();

    assert!(
        html.contains("<pre class=\"rpv-code\"><code>use dioxus::prelude::*;"),
        "{html}"
    );
    assert!(!html.contains("class=\"dxc"), "{html}");
}

#[test]
fn code_block_renders_plain_text_with_an_optional_label() {
    let labelled = dioxus_ssr::render_element(rsx! {
        CodeBlock { content: "dx components add button", label: "shell" }
    });
    let unlabelled = dioxus_ssr::render_element(rsx! {
        CodeBlock { content: String::from("[components]") }
    });

    assert_eq!(
        labelled,
        "<figure class=\"rpv-code-panel\"><figcaption class=\"rpv-code-panel__label\">shell</figcaption><pre class=\"rpv-code\"><code>dx components add button</code></pre></figure>"
    );
    assert_eq!(
        unlabelled,
        "<figure class=\"rpv-code-panel\"><pre class=\"rpv-code\"><code>[components]</code></pre></figure>"
    );
}

#[cfg(feature = "syntax-highlighting")]
#[test]
fn code_block_renders_consumer_supplied_highlighted_source() {
    use dioxus_registry_preview::code::{HighlightSpan, HighlightedSource, Language};

    const SPANS: &[HighlightSpan] = &[HighlightSpan::new(0..3, "k")];
    let html = dioxus_ssr::render_element(rsx! {
        CodeBlock {
            content: HighlightedSource::from_static_parts("let x = 1;", Language::Rust, SPANS),
            label: "snippet.rs",
        }
    });

    assert!(
        html.contains("<figcaption class=\"rpv-code-panel__label\">snippet.rs</figcaption>"),
        "{html}"
    );
    assert!(
        html.contains("<pre class=\"dxc dxc-github-dark\" data-language=\"rust\"><code>"),
        "{html}"
    );
    assert!(html.contains("<span class=\"a-k\">let</span>"), "{html}");
}
