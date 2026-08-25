//! Dioxus-free discovery, validation, and Markdown rendering for Component Registries.
//!
//! Use this crate when implementing build tooling around Registry authoring files. Applications
//! should normally depend on `dioxus-registry-preview`, which re-exports the supported validation
//! surface alongside the macros and default Dioxus chrome.
//!
//! # Validate a site
//!
//! ```no_run
//! use std::path::Path;
//!
//! use dioxus_registry_preview_core::{MarkdownOptions, load_site_from_catalog};
//!
//! let validation = load_site_from_catalog(
//!     Path::new("src/preview/pages.rs"),
//!     &MarkdownOptions::default(),
//! );
//! for diagnostic in &validation.diagnostics {
//!     eprintln!("{diagnostic}");
//! }
//! assert!(validation.diagnostics.is_empty());
//! ```
//!
//! Validation paths are resolved from the source file containing `component_pages!`. Site loading
//! aggregates independent diagnostics and may retain a partial model for Consumer-owned policy.
//!
//! # Trusted Markdown
//!
//! Markdown rendering preserves raw HTML and accepts trusted replacement HTML from
//! [`MarkdownHooks`]. Sanitize untrusted authoring input before rendering it.

mod diagnostic;
mod loading;
mod markdown;
mod syntax;

pub use diagnostic::{Diagnostic, DiagnosticCode, DiagnosticTarget, Validation};
pub use loading::{
    CatalogDocumentation, CatalogEntry, ComponentDocumentation, ComponentReadme,
    DocumentationGroup, DocumentationPage, DocumentationSite, ExampleDocumentation, PageKind,
    RegistrySource, component_invocation, load_catalog, load_component,
    load_component_with_markdown, load_site_from_catalog, readme_parts, validate_registry_source,
};
pub use markdown::{MarkdownHooks, MarkdownOptions, markdown_to_html};
pub use syntax::{
    CatalogInvocation, ComponentInvocation, ExampleInvocation, GroupInvocation, GroupMapping,
    PageCatalogInvocation, example_section_name, sentence_case,
};
