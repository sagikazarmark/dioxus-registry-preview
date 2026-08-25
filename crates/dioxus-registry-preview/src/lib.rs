//! Convention-driven tooling for Dioxus Component Registry Preview applications.
//!
//! This facade is the Consumer entry point for authoring macros, Dioxus 0.7 default chrome, and
//! Dioxus-free full-site [`validation`]. It supports Rust 1.88 and later.
//!
//! Invoke [`component!`] from each Component's `docs/mod.rs`, then invoke [`component_pages!`]
//! once beside the Preview's page catalog. The generated pieces can be composed into a custom
//! application or passed to [`chrome::App`] for the complete default site.
//!
//! # Validate authoring files
//!
//! ```no_run
//! use std::path::Path;
//!
//! use dioxus_registry_preview::validation::{MarkdownOptions, load_site_from_catalog};
//!
//! let validation = load_site_from_catalog(
//!     Path::new("src/preview/pages.rs"),
//!     &MarkdownOptions::default(),
//! );
//! assert!(validation.diagnostics.is_empty());
//! ```
//!
//! See the repository's Registry author guide for the companion manifests, README, and Example
//! files required by the macros.

pub mod chrome;

pub use dioxus_registry_preview_macros::{component, component_pages};

/// Full-site discovery and validation for Consumer tests and other Rust tooling.
pub mod validation {
    pub use dioxus_registry_preview_core::{
        CatalogDocumentation, CatalogEntry, CatalogInvocation, ComponentDocumentation,
        ComponentInvocation, ComponentReadme, Diagnostic, DiagnosticCode, DiagnosticTarget,
        DocumentationGroup, DocumentationPage, DocumentationSite, ExampleDocumentation,
        ExampleInvocation, GroupInvocation, GroupMapping, MarkdownHooks, MarkdownOptions,
        PageCatalogInvocation, PageKind, RegistrySource, Validation, component_invocation,
        example_section_name, load_catalog, load_component, load_component_with_markdown,
        load_site_from_catalog, markdown_to_html, readme_parts, sentence_case,
        validate_registry_source,
    };
}

/// Where a page appears when it is listed in site navigation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum NavigationPlacement<Group> {
    /// A page listed under a standalone navigation heading.
    Standalone(&'static str),
    /// A generated Component page listed in a Component group.
    Group(Group),
}

/// Whether a page appears in site navigation and indexes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum ListingPolicy {
    /// Include the page in navigation and indexes.
    Listed,
    /// Keep the page routable while omitting it from navigation and indexes.
    Unlisted,
}

impl ListingPolicy {
    /// Converts the boolean authored by `component!` into a catalog policy.
    #[must_use]
    pub const fn from_listed(listed: bool) -> Self {
        if listed { Self::Listed } else { Self::Unlisted }
    }

    /// Whether this policy includes the page in navigation and indexes.
    #[must_use]
    pub const fn is_listed(self) -> bool {
        matches!(self, Self::Listed)
    }

    /// The stable value rendered in the DOM catalog manifest.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Listed => "listed",
            Self::Unlisted => "unlisted",
        }
    }
}

/// Whether generic browser tooling should visit a page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum BrowserTestPolicy {
    /// Include the page in generic browser coverage.
    Enabled,
    /// Omit the page from generic browser coverage.
    Disabled,
}

impl BrowserTestPolicy {
    /// Whether generic browser tooling should visit this page.
    #[must_use]
    pub const fn is_enabled(self) -> bool {
        matches!(self, Self::Enabled)
    }

    /// The stable value rendered in the DOM catalog manifest.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Enabled => "enabled",
            Self::Disabled => "disabled",
        }
    }
}

/// One page in a documentation site's complete ordered catalog.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct PageDescriptor<Group, Kind> {
    /// The nonempty stable ID written to `data-catalog-page` and `data-page`.
    pub id: &'static str,
    /// The unique absolute route path.
    pub path: &'static str,
    /// The page title used by navigation and document titles.
    pub title: &'static str,
    /// The page summary used by indexes and the DOM catalog.
    pub description: &'static str,
    /// The page's navigation section or Component group.
    pub navigation: NavigationPlacement<Group>,
    /// Whether navigation and indexes list the page.
    pub listing: ListingPolicy,
    /// Whether generic browser tooling visits the page.
    pub browser_tests: BrowserTestPolicy,
    /// The Consumer-owned rendering value.
    pub kind: Kind,
}

impl<Group, Kind> PageDescriptor<Group, Kind> {
    /// Creates one authored page descriptor.
    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub const fn new(
        id: &'static str,
        path: &'static str,
        title: &'static str,
        description: &'static str,
        navigation: NavigationPlacement<Group>,
        listing: ListingPolicy,
        browser_tests: BrowserTestPolicy,
        kind: Kind,
    ) -> Self {
        Self {
            id,
            path,
            title,
            description,
            navigation,
            listing,
            browser_tests,
            kind,
        }
    }
}

/// Documentation derived from one registry component's authoring files.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ComponentDocumentation {
    /// The component's install name from its manifest.
    pub name: &'static str,
    /// The page title from the README's H1.
    pub title: &'static str,
    /// The component's catalog description from its manifest.
    pub description: &'static str,
    /// The complete README as it appears on GitHub.
    pub readme: &'static str,
    /// The README body rendered below the Preview's existing heading and lead.
    pub readme_html: &'static str,
    /// The examples in page order.
    pub examples: &'static [ExampleDocumentation],
}

impl ComponentDocumentation {
    #[doc(hidden)]
    #[must_use]
    pub const fn new(
        name: &'static str,
        title: &'static str,
        description: &'static str,
        readme: &'static str,
        readme_html: &'static str,
        examples: &'static [ExampleDocumentation],
    ) -> Self {
        Self {
            name,
            title,
            description,
            readme,
            readme_html,
            examples,
        }
    }
}

/// Documentation for one compiling, rendered example.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ExampleDocumentation {
    /// The URL fragment and browser-test identifier.
    pub slug: &'static str,
    /// The example heading.
    pub title: &'static str,
    /// What the example demonstrates.
    pub description: &'static str,
    /// The exact source file that is compiled and rendered.
    pub source: &'static str,
}

impl ExampleDocumentation {
    #[doc(hidden)]
    #[must_use]
    pub const fn new(
        slug: &'static str,
        title: &'static str,
        description: &'static str,
        source: &'static str,
    ) -> Self {
        Self {
            slug,
            title,
            description,
            source,
        }
    }
}
