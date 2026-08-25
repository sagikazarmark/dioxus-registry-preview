//! Default documentation chrome with isolated, dependency-owned styling.

use std::{borrow::Cow, collections::HashSet, convert::Infallible, fmt, rc::Rc, str::FromStr};

use dioxus::prelude::*;
use dioxus::router::{SegmentType, SiteMapSegment};

use crate::{ExampleDocumentation, NavigationPlacement, PageDescriptor};

const STYLESHEET: &str = include_str!("chrome.css");
const APP_STYLESHEET: &str = include_str!("app.css");
const SIDEBAR_STYLESHEET: &str = include_str!("sidebar.css");
const THEME_SWITCHER_SCRIPT: &str = include_str!("theme-switcher.js");
const DEFAULT_HOME_ID: &str = "registry-home";
const DEFAULT_INSTALLATION_ID: &str = "registry-installation";
const DEFAULT_INSTALLATION_PATH: &str = "/installation";
const DEFAULT_THEMES: [(ThemeEntry, &str); 2] = [
    (ThemeEntry::new("light", true, true), "Light"),
    (ThemeEntry::new("dark", true, false), "Dark"),
];

/// One page exposed by a Consumer's rendered catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct CatalogEntry {
    /// The nonempty stable page ID.
    pub id: &'static str,
    /// The unique absolute route path.
    pub path: &'static str,
    /// The page title.
    pub title: &'static str,
    /// The page summary.
    pub description: &'static str,
    /// The protocol rendering kind: `generated-component` or `rsx`.
    pub kind: &'static str,
    /// The protocol navigation placement: `standalone` or `group`.
    pub navigation: &'static str,
    /// The standalone heading or stable Component group ID.
    pub navigation_value: &'static str,
    /// The protocol listing policy: `listed` or `unlisted`.
    pub listing: &'static str,
    /// The protocol browser policy: `enabled` or `disabled`.
    pub browser_test: &'static str,
}

impl CatalogEntry {
    /// Creates one page entry for the rendered DOM catalog.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        id: &'static str,
        path: &'static str,
        title: &'static str,
        description: &'static str,
        kind: &'static str,
        navigation: &'static str,
        navigation_value: &'static str,
        listing: &'static str,
        browser_test: &'static str,
    ) -> Self {
        Self {
            id,
            path,
            title,
            description,
            kind,
            navigation,
            navigation_value,
            listing,
            browser_test,
        }
    }
}

/// One theme exposed through the DOM protocol.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ThemeEntry {
    /// The nonempty stable theme ID.
    pub id: &'static str,
    /// Whether generic screenshots cover this theme.
    pub baseline: bool,
    /// Whether this theme is initially selected.
    pub checked: bool,
}

impl ThemeEntry {
    /// Creates one theme entry for the rendered DOM manifest.
    #[must_use]
    pub const fn new(id: &'static str, baseline: bool, checked: bool) -> Self {
        Self {
            id,
            baseline,
            checked,
        }
    }
}

/// The Git source used by generated Registry installation instructions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegistrySource {
    repository: &'static str,
    revision: Option<&'static str>,
}

impl RegistrySource {
    /// Creates an unpinned Git source.
    #[must_use]
    pub const fn git(repository: &'static str) -> Self {
        Self {
            repository,
            revision: None,
        }
    }

    /// Pins installation commands and configuration to one Git revision.
    #[must_use]
    pub const fn with_revision(self, revision: &'static str) -> Self {
        Self {
            revision: Some(revision),
            ..self
        }
    }

    /// The Registry's clone URL.
    #[must_use]
    pub const fn repository(self) -> &'static str {
        self.repository
    }

    /// The optional Git revision used by installation instructions.
    #[must_use]
    pub const fn revision(self) -> Option<&'static str> {
        self.revision
    }
}

/// Registry facts used by the generated installation page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RegistryDocumentation {
    name: &'static str,
    description: &'static str,
    components: &'static [&'static str],
    default_component: &'static str,
    source: RegistrySource,
}

impl RegistryDocumentation {
    #[doc(hidden)]
    #[must_use]
    pub const fn new(
        name: &'static str,
        description: &'static str,
        components: &'static [&'static str],
        default_component: &'static str,
        source: RegistrySource,
    ) -> Self {
        Self {
            name,
            description,
            components,
            default_component,
            source,
        }
    }

    /// The name from the root Registry manifest.
    #[must_use]
    pub const fn name(self) -> &'static str {
        self.name
    }

    /// The description from the root Registry manifest.
    #[must_use]
    pub const fn description(self) -> &'static str {
        self.description
    }

    /// Component install names in root Registry-manifest order.
    #[must_use]
    pub const fn components(self) -> &'static [&'static str] {
        self.components
    }

    /// The install name selected by `component_pages!`'s `default` module.
    #[must_use]
    pub const fn default_component(self) -> &'static str {
        self.default_component
    }

    /// The source used by generated installation instructions.
    #[must_use]
    pub const fn source(self) -> RegistrySource {
        self.source
    }
}

/// A generated page catalog paired with its group-ID mapping and Registry facts.
pub struct AppCatalog<Descriptor: 'static> {
    pages: &'static [Descriptor],
    group_id: fn(&Descriptor) -> &'static str,
    description: &'static str,
    registry: Option<RegistryDocumentation>,
}

impl<Descriptor> AppCatalog<Descriptor> {
    #[doc(hidden)]
    #[must_use]
    pub const fn new(
        pages: &'static [Descriptor],
        group_id: fn(&Descriptor) -> &'static str,
    ) -> Self {
        Self {
            pages,
            group_id,
            description: "",
            registry: None,
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn with_description(self, description: &'static str) -> Self {
        Self {
            description,
            ..self
        }
    }

    #[doc(hidden)]
    #[must_use]
    pub const fn with_registry(self, registry: RegistryDocumentation) -> Self {
        Self {
            registry: Some(registry),
            ..self
        }
    }

    /// The complete descriptor slice emitted by `component_pages!`.
    #[must_use]
    pub const fn pages(self) -> &'static [Descriptor] {
        self.pages
    }

    /// The description from the root Registry manifest.
    #[must_use]
    pub const fn description(self) -> &'static str {
        self.description
    }

    /// Registry facts emitted by `component_pages!` for installation presentation.
    #[must_use]
    pub const fn registry(self) -> Option<RegistryDocumentation> {
        self.registry
    }

    /// Resolves one Component group through the catalog invocation's stable mapping.
    ///
    /// # Panics
    ///
    /// The generated mapping panics when a handwritten grouped descriptor uses a Consumer group
    /// value that has no stable ID in the `component_pages!` invocation.
    pub fn group_id(self, page: &Descriptor) -> &'static str {
        (self.group_id)(page)
    }
}

impl<Descriptor> Clone for AppCatalog<Descriptor> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<Descriptor> Copy for AppCatalog<Descriptor> {}

impl<Descriptor> PartialEq for AppCatalog<Descriptor> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.pages, other.pages)
            && std::ptr::fn_addr_eq(self.group_id, other.group_id)
            && self.description == other.description
            && self.registry == other.registry
    }
}

impl<Descriptor> Eq for AppCatalog<Descriptor> {}

impl<Descriptor: fmt::Debug> fmt::Debug for AppCatalog<Descriptor> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("AppCatalog")
            .field("pages", &self.pages)
            .field("description", &self.description)
            .field("registry", &self.registry)
            .finish_non_exhaustive()
    }
}

/// A Consumer-owned rendering kind that can be shown by [`App`].
pub trait AppPage: Clone + Copy + PartialEq + 'static {
    /// The stable rendering-kind name written to the DOM protocol.
    ///
    /// Protocol version 1 accepts `generated-component` and `rsx`.
    fn kind(self) -> &'static str;

    /// Renders this page through the Consumer's literal dispatch.
    ///
    /// # Errors
    ///
    /// Returns a Dioxus rendering error when the Consumer's page cannot be rendered.
    fn render(self) -> Element;
}

#[derive(Clone)]
struct AppCatalogEntry {
    descriptor: CatalogEntry,
    render: Rc<dyn Fn(&str) -> Element>,
}

#[derive(Clone)]
struct AppContext {
    title: &'static str,
    catalog: Vec<CatalogEntry>,
    pages: Vec<AppCatalogEntry>,
}

#[derive(Clone, Debug, PartialEq)]
struct AppRoute(String);

impl FromStr for AppRoute {
    type Err = Infallible;

    fn from_str(route: &str) -> Result<Self, Self::Err> {
        Ok(Self(String::from(route)))
    }
}

impl fmt::Display for AppRoute {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl Routable for AppRoute {
    const SITE_MAP: &'static [SiteMapSegment] = &[SiteMapSegment {
        segment_type: SegmentType::CatchAll("path"),
        children: &[],
    }];

    fn render(&self, level: usize) -> Element {
        if level == 0 {
            rsx! { AppRouteView { route: self.0.clone() } }
        } else {
            VNode::empty()
        }
    }
}

/// Properties for the batteries-included [`App`].
#[derive(Clone, PartialEq, Props)]
pub struct AppProps<Group: Clone + Copy + PartialEq + 'static, Page: AppPage> {
    /// The catalog, group-ID mapping, and Registry description emitted by `component_pages!`.
    pub catalog: AppCatalog<PageDescriptor<Group, Page>>,
    /// The Registry name shown in the header and appended to document titles.
    #[props(default = "Registry documentation")]
    pub title: &'static str,
}

/// A batteries-included documentation site derived from one page catalog.
///
/// The app supplies dynamic routing, document titles, default navigation and themes, a not-found
/// page, and all DOM protocol writers. It synthesizes `registry-installation` and `registry-home`
/// when the authored catalog does not claim `/installation` and `/`, respectively.
///
/// # Panics
///
/// Panics when an authored descriptor uses an ID reserved by an active synthesized page, when a
/// handwritten Component group has no stable group ID, or when the resulting catalog violates the
/// DOM protocol invariants documented by [`PageCatalogManifest`].
///
/// # Errors
///
/// Returns a Dioxus rendering error when the router cannot be rendered.
// Dioxus component functions conventionally consume their generated props value.
#[allow(non_snake_case)]
#[allow(clippy::needless_pass_by_value, clippy::too_many_lines)]
pub fn App<Group: Clone + Copy + PartialEq + 'static, Page: AppPage>(
    props: AppProps<Group, Page>,
) -> Element {
    let mut pages = props
        .catalog
        .pages()
        .iter()
        .map(|page| {
            let navigation = match page.navigation {
                NavigationPlacement::Standalone(section) => ("standalone", section),
                NavigationPlacement::Group(_) => ("group", props.catalog.group_id(page)),
            };
            let descriptor = CatalogEntry::new(
                page.id,
                page.path,
                page.title,
                page.description,
                page.kind.kind(),
                navigation.0,
                navigation.1,
                page.listing.as_str(),
                page.browser_tests.as_str(),
            );
            let kind = page.kind;

            AppCatalogEntry {
                descriptor,
                render: Rc::new(move |_| kind.render()),
            }
        })
        .collect::<Vec<_>>();

    if let Some(registry) = props.catalog.registry()
        && !pages
            .iter()
            .any(|page| page.descriptor.path == DEFAULT_INSTALLATION_PATH)
    {
        assert!(
            !pages
                .iter()
                .any(|page| page.descriptor.id == DEFAULT_INSTALLATION_ID),
            "`registry-installation` is reserved by chrome::App when it supplies the installation page"
        );
        pages.insert(
            0,
            AppCatalogEntry {
                descriptor: CatalogEntry::new(
                    DEFAULT_INSTALLATION_ID,
                    DEFAULT_INSTALLATION_PATH,
                    "Installation",
                    "Install components from this Registry.",
                    "rsx",
                    "standalone",
                    "Getting started",
                    "listed",
                    "enabled",
                ),
                render: Rc::new(move |_| rsx! { InstallationPage { registry } }),
            },
        );
    }

    if !pages.iter().any(|page| page.descriptor.path == "/") {
        assert!(
            !pages
                .iter()
                .any(|page| page.descriptor.id == DEFAULT_HOME_ID),
            "`registry-home` is reserved by chrome::App when it supplies the default home"
        );
        let authored_catalog = pages.iter().map(|page| page.descriptor).collect::<Vec<_>>();
        let call_to_action = authored_catalog
            .iter()
            .find(|page| page.path == DEFAULT_INSTALLATION_PATH)
            .copied()
            .or_else(|| authored_catalog.first().copied());
        let home_catalog = authored_catalog.clone();
        let title = props.title;
        let description = props.catalog.description();

        pages.insert(
            0,
            AppCatalogEntry {
                descriptor: CatalogEntry::new(
                    DEFAULT_HOME_ID,
                    "/",
                    "Home",
                    description,
                    "rsx",
                    "standalone",
                    "Getting started",
                    "unlisted",
                    "enabled",
                ),
                render: Rc::new(move |route_query| {
                    rsx! {
                        DefaultHome {
                            title,
                            description,
                            catalog: home_catalog.clone(),
                            call_to_action,
                            route_query: String::from(route_query),
                        }
                    }
                }),
            },
        );
    }
    let catalog = pages.iter().map(|page| page.descriptor).collect();

    use_context_provider(move || AppContext {
        title: props.title,
        catalog,
        pages,
    });

    rsx! {
        document::Style { "{APP_STYLESHEET}" }
        Router::<AppRoute> {}
    }
}

/// Installation instructions generated from the root Registry and catalog invocation.
#[component]
pub fn InstallationPage(registry: RegistryDocumentation) -> Element {
    let source = registry.source();
    let repository = shell_argument(source.repository());
    let revision_argument = source
        .revision()
        .map(|revision| format!(" --rev {}", shell_argument(revision)))
        .unwrap_or_default();
    let component_command = format!(
        "dx components add {} --git {repository}{revision_argument}",
        registry.default_component()
    );
    let all_command = format!("dx components add --all --git {repository}{revision_argument}");
    let repository = toml_string(source.repository());
    let configuration = source.revision().map_or_else(
        || format!("[components]\nregistry = {{ git = \"{repository}\" }}"),
        |revision| {
            let revision = toml_string(revision);
            format!("[components]\nregistry = {{ git = \"{repository}\", rev = \"{revision}\" }}")
        },
    );

    rsx! {
        section { class: "rpv-installation",
            header { class: "rpv-installation__header",
                h1 { class: "rpv-installation__title", "Install {registry.name()}" }
                if !registry.description().is_empty() {
                    p { class: "rpv-installation__description", "{registry.description()}" }
                }
            }
            section { class: "rpv-installation__section",
                h2 { class: "rpv-installation__section-title", "Add a component" }
                p { class: "rpv-installation__copy",
                    "Install the default component directly from the Registry's Git source."
                }
                CodeBlock { source: component_command }
            }
            section { class: "rpv-installation__section",
                h2 { class: "rpv-installation__section-title", "Add every component" }
                CodeBlock { source: all_command }
            }
            section { class: "rpv-installation__section",
                h2 { class: "rpv-installation__section-title", "Configure Dioxus" }
                p { class: "rpv-installation__copy",
                    "Add the Registry to Dioxus.toml so later commands only need a component name."
                }
                CodeBlock { source: configuration }
            }
            section { class: "rpv-installation__section",
                h2 { class: "rpv-installation__section-title", "Available components" }
                ul { class: "rpv-installation__components",
                    for component in registry.components() {
                        li {
                            code { class: "rpv-installation__component", "{component}" }
                        }
                    }
                }
            }
        }
    }
}

fn shell_argument(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\"'\"'"))
}

fn toml_string(value: &str) -> String {
    value.replace('\\', "\\\\").replace('"', "\\\"")
}

#[component]
fn AppRouteView(route: String) -> Element {
    let context = consume_context::<AppContext>();
    let router = router();
    let current_route = route
        .split_once('#')
        .map_or(route.as_str(), |route| route.0);
    let (path, route_query) = current_route.split_once('?').unwrap_or((current_route, ""));
    let path = if path.is_empty() { "/" } else { path };
    let current_page = context
        .pages
        .iter()
        .find(|page| page.descriptor.path == path)
        .cloned();
    let page_id = current_page
        .as_ref()
        .map(|page| String::from(page.descriptor.id));
    let document_title = current_page.as_ref().map_or_else(
        || format!("Page not found | {}", context.title),
        |page| format!("{} | {}", page.descriptor.title, context.title),
    );
    let page = current_page.map(|page| (page.render)(route_query));
    let href_query = String::from(route_query);
    let navigate_query = String::from(route_query);
    let brand_href = app_href("/", route_query);

    rsx! {
        document::Title { "{document_title}" }
        Shell {
            page_id: page_id.clone(),
            catalog: context.catalog.clone(),
            header { class: "rpv-header",
                a {
                    class: "rpv-header__brand",
                    href: brand_href,
                    "{context.title}"
                }
            }
            div { class: "rpv-layout",
                Sidebar {
                    page_id,
                    catalog: context.catalog.clone(),
                    href_for: move |page: CatalogEntry| {
                        app_href(page.path, &href_query)
                    },
                    navigate: move |page: CatalogEntry| {
                        router.push(app_href(page.path, &navigate_query));
                    },
                }
                article { class: "rpv-page",
                    if let Some(page) = page {
                        {page}
                    } else {
                        h1 { class: "rpv-not-found__title", "Page not found" }
                        p { class: "rpv-not-found__description",
                            "No documentation page matches this address."
                        }
                    }
                }
            }
        }
    }
}

#[component]
fn DefaultHome(
    title: &'static str,
    description: &'static str,
    catalog: Vec<CatalogEntry>,
    call_to_action: Option<CatalogEntry>,
    route_query: String,
) -> Element {
    let sections = home_catalog_sections(&catalog);

    rsx! {
        section { class: "rpv-home",
            header { class: "rpv-home__header",
                h1 { class: "rpv-home__title", "{title}" }
                if !description.is_empty() {
                    p { class: "rpv-home__description", "{description}" }
                }
                if let Some(page) = call_to_action {
                    a {
                        class: "rpv-home__cta",
                        href: app_href(page.path, &route_query),
                        "Getting started"
                    }
                }
            }
            div { class: "rpv-home__index",
                for (placement, section, pages) in sections {
                    section { key: "{placement}-{section}", class: "rpv-home__section",
                        h2 { class: "rpv-home__section-title", "{section}" }
                        ul { class: "rpv-home__list",
                            for page in pages {
                                li { key: "{page.id}", class: "rpv-home__entry",
                                    a {
                                        class: "rpv-home__link",
                                        href: app_href(page.path, &route_query),
                                        span { class: "rpv-home__entry-title", "{page.title}" }
                                        span { class: "rpv-home__entry-description", "{page.description}" }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

fn home_catalog_sections(catalog: &[CatalogEntry]) -> Vec<CatalogSection> {
    let mut sections = Vec::new();

    for placement in ["standalone", "group"] {
        for page in catalog
            .iter()
            .copied()
            .filter(|page| page.listing == "listed" && page.navigation == placement)
        {
            push_catalog_page(&mut sections, page);
        }
    }

    sections
}

type CatalogSection = (&'static str, &'static str, Vec<CatalogEntry>);

fn push_catalog_page(sections: &mut Vec<CatalogSection>, page: CatalogEntry) {
    if let Some((_, _, pages)) = sections.iter_mut().find(|(placement, value, _)| {
        *placement == page.navigation && *value == page.navigation_value
    }) {
        pages.push(page);
    } else {
        sections.push((page.navigation, page.navigation_value, vec![page]));
    }
}

fn app_href(path: &str, query: &str) -> String {
    if query.is_empty() {
        String::from(path)
    } else {
        format!("{path}?{query}")
    }
}

/// The default chrome around one rendered Example and its source.
#[component]
pub fn ExampleSection(documentation: ExampleDocumentation, children: Element) -> Element {
    rsx! {
        section { "data-example": documentation.slug, class: "rpv-example",
            h2 { class: "rpv-example__title", "{documentation.title}" }
            p { class: "rpv-example__description", "{documentation.description}" }
            div { "data-example-content": "true", class: "rpv-example__content", {children} }
            CodeBlock { source: documentation.source }
        }
    }
}

/// The default chrome around trusted, build-time README HTML.
///
/// This component inserts `html` through `dangerous_inner_html` without sanitization. Callers must
/// pass only trusted build-time output or sanitize untrusted content before calling it.
#[component]
pub fn ReadmeSection(html: &'static str) -> Element {
    rsx! {
        section {
            "data-readme": "true",
            class: "rpv-readme",
            dangerous_inner_html: html,
        }
    }
}

/// Code presentation used by default chrome components.
#[component]
pub fn CodeBlock(#[props(into)] source: Cow<'static, str>) -> Element {
    rsx! {
        figure { class: "rpv-code-panel",
            figcaption { class: "rpv-code-panel__label", "Source" }
            pre { class: "rpv-code",
                code { "{source}" }
            }
        }
    }
}

/// Installs the isolated default chrome stylesheet into the document head.
#[component]
pub fn Styles() -> Element {
    rsx! { document::Style { "{STYLESHEET}" } }
}

/// Writes the complete page catalog consumed by generic browser tooling.
///
/// # Panics
///
/// Panics when IDs are empty or duplicated, paths are relative or duplicated, or a protocol value
/// is unsupported or missing.
#[component]
pub fn PageCatalogManifest(catalog: Vec<CatalogEntry>) -> Element {
    validate_catalog(&catalog);

    rsx! {
        div { "data-page-catalog": "true", hidden: true,
            for page in catalog {
                span {
                    key: "{page.id}",
                    "data-catalog-page": page.id,
                    "data-path": page.path,
                    "data-title": page.title,
                    "data-description": page.description,
                    "data-kind": page.kind,
                    "data-navigation": page.navigation,
                    "data-navigation-value": page.navigation_value,
                    "data-listing": page.listing,
                    "data-browser-test": page.browser_test,
                }
            }
        }
    }
}

/// Writes the available themes consumed by generic browser tooling.
///
/// # Panics
///
/// Panics when the collection is empty, IDs are empty or duplicated, no baseline exists, or the
/// collection does not contain exactly one initially checked theme.
#[component]
pub fn ThemeSwitcherManifest(themes: Vec<ThemeEntry>) -> Element {
    validate_themes(&themes);

    rsx! {
        div { "data-switcher": "theme", hidden: true,
            for theme in themes {
                input {
                    key: "{theme.id}",
                    r#type: "radio",
                    name: "documentation-theme",
                    value: theme.id,
                    checked: theme.checked,
                    "data-value": theme.id,
                    "data-baseline": theme.baseline.then_some("true"),
                }
            }
        }
    }
}

fn validate_catalog(catalog: &[CatalogEntry]) {
    let mut ids = HashSet::new();
    let mut paths = HashSet::new();

    for page in catalog {
        assert!(!page.id.is_empty(), "page catalog IDs must be nonempty");
        assert!(
            ids.insert(page.id),
            "page catalog contains duplicate ID `{}`",
            page.id
        );
        assert!(
            page.path.starts_with('/'),
            "page catalog path `{}` must be absolute",
            page.path
        );
        assert!(
            paths.insert(page.path),
            "page catalog contains duplicate path `{}`",
            page.path
        );
        assert!(
            matches!(page.kind, "generated-component" | "rsx"),
            "page `{}` has unsupported rendering kind `{}`",
            page.id,
            page.kind
        );
        assert!(
            matches!(page.navigation, "standalone" | "group"),
            "page `{}` has unsupported navigation placement `{}`",
            page.id,
            page.navigation
        );
        assert!(
            !page.navigation_value.is_empty(),
            "page `{}` must have a nonempty navigation value",
            page.id
        );
        assert!(
            matches!(page.listing, "listed" | "unlisted"),
            "page `{}` has unsupported listing policy `{}`",
            page.id,
            page.listing
        );
        assert!(
            matches!(page.browser_test, "enabled" | "disabled"),
            "page `{}` has unsupported browser-test policy `{}`",
            page.id,
            page.browser_test
        );
    }
}

fn validate_themes(themes: &[ThemeEntry]) {
    assert!(!themes.is_empty(), "theme manifest must not be empty");

    let mut ids = HashSet::new();
    let mut baseline_count = 0;
    let mut checked_count = 0;

    for theme in themes {
        assert!(!theme.id.is_empty(), "theme IDs must be nonempty");
        assert!(
            ids.insert(theme.id),
            "theme manifest contains duplicate ID `{}`",
            theme.id
        );
        baseline_count += usize::from(theme.baseline);
        checked_count += usize::from(theme.checked);
    }

    assert!(
        baseline_count > 0,
        "theme manifest must contain at least one baseline theme"
    );
    assert_eq!(
        checked_count, 1,
        "theme manifest must contain exactly one checked theme"
    );
}

/// The default visible light and dark theme control.
#[component]
pub fn ThemeSwitcher() -> Element {
    rsx! {
        document::Script { "{THEME_SWITCHER_SCRIPT}" }

        ThemeSwitcherManifest {
            themes: DEFAULT_THEMES.iter().map(|(theme, _)| *theme).collect(),
        }

        div {
            class: "rpv-theme-switcher",
            role: "group",
            aria_label: "Color theme",
            span { class: "rpv-theme-switcher__label", "Theme" }
            for (theme, label) in DEFAULT_THEMES {
                label {
                    key: "{theme.id}",
                    class: "rpv-theme-switcher__option",
                    input {
                        r#type: "radio",
                        name: "documentation-theme-control",
                        value: theme.id,
                        checked: theme.checked,
                        class: "rpv-theme-switcher__control",
                    }
                    span { class: "rpv-theme-switcher__option-label", "{label}" }
                }
            }
        }
    }
}

/// Visible navigation derived from a Consumer's ordered page catalog.
///
/// `href_for` supplies every link target. When `navigate` is present, unmodified primary clicks are
/// intercepted and forwarded to it; modified and non-primary clicks retain normal browser behavior.
#[component]
pub fn Sidebar(
    page_id: Option<String>,
    catalog: Vec<CatalogEntry>,
    href_for: Callback<CatalogEntry, String>,
    #[props(default)] navigate: Option<Callback<CatalogEntry>>,
) -> Element {
    let mut sections = Vec::new();

    for page in catalog.into_iter().filter(|page| page.listing == "listed") {
        if !matches!(page.navigation, "standalone" | "group") {
            continue;
        }

        push_catalog_page(&mut sections, page);
    }

    rsx! {
        document::Style { "{SIDEBAR_STYLESHEET}" }

        nav {
            "data-switcher": "component",
            class: "rpv-sidebar",
            aria_label: "Documentation",
            for (placement, section, pages) in sections {
                section { key: "{placement}-{section}", class: "rpv-sidebar__section",
                    h2 { class: "rpv-sidebar__heading", "{section}" }
                    ul { class: "rpv-sidebar__list",
                        for page in pages {
                            li { key: "{page.id}",
                                a {
                                    "data-value": (placement == "group").then_some(page.id),
                                    class: "rpv-sidebar__link",
                                    href: href_for.call(page),
                                    aria_current: (page_id.as_deref() == Some(page.id)).then_some("page"),
                                    onclick: move |event| {
                                        if let Some(navigate) = navigate {
                                            if !event.modifiers().is_empty()
                                                || event.trigger_button()
                                                    != Some(dioxus::html::input_data::MouseButton::Primary)
                                            {
                                                return;
                                            }
                                            event.prevent_default();
                                            navigate.call(page);
                                        }
                                    },
                                    "{page.title}"
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Default site chrome and DOM protocol writers.
///
/// An empty `themes` collection mounts the visible default light/dark switcher. A nonempty
/// collection writes only the Consumer-provided hidden theme manifest.
#[component]
pub fn Shell(
    page_id: Option<String>,
    catalog: Vec<CatalogEntry>,
    #[props(default)] themes: Vec<ThemeEntry>,
    children: Element,
) -> Element {
    let uses_default_themes = themes.is_empty();

    rsx! {
        Styles {}

        PageCatalogManifest { catalog }

        if !uses_default_themes {
            ThemeSwitcherManifest { themes }
        }

        main {
            "data-registry-preview-chrome": "true",
            "data-page": page_id,
            class: "rpv-shell",
            if uses_default_themes {
                ThemeSwitcher {}
            }
            {children}
        }
    }
}
