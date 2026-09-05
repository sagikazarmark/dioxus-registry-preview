//! Procedural macro implementation for `dioxus-registry-preview`.
//!
//! Consumers should depend on the facade and use its [`component!`](macro@component) and
//! [`component_pages!`](macro@component_pages) re-exports. This package is versioned and released
//! with the facade but is not a direct integration surface.

use std::path::PathBuf;

use dioxus_registry_preview_core::{
    CatalogInvocation, ComponentInvocation, Diagnostic, DiagnosticTarget, example_section_name,
    load_catalog, load_component, validate_registry_source,
};
use proc_macro::TokenStream;
use proc_macro_crate::{FoundCrate, crate_name};
use proc_macro2::Span;
use quote::quote;
use syn::{Ident, LitStr, Result, parse_macro_input};

#[cfg(feature = "syntax-highlighting")]
mod highlight;

/// Generates one component's documentation metadata and Preview page.
///
/// The macro is invoked from `<component>/docs/mod.rs`. By default it reads
/// `../component.json`, `../component.rs`, `../README.md`, and the declared
/// examples under `examples/`.
///
/// The invocation accepts `component_root`, `group`, `example_section`, `readme_section`,
/// `render_readme`, `listed`, `page`, and `examples`. Only `group` and `examples` are required.
/// Example entries require `description` and may override `slug` and `title`. Raw Example module
/// identifiers are rejected because generated section names and source paths use their ordinary
/// identifier spelling.
///
/// The macro emits `GROUP_ID`, `LISTED`, `DOCUMENTATION`, one public section Component per Example,
/// an optional `ReadmeSection`, and `DocumentationPage`. A `page` override replaces only the
/// default page assembly; all composable generated pieces remain available.
///
/// Files are discovered relative to the physical invocation source through
/// `proc_macro::Span::local_file()`. Authoring and validation failures are emitted as compile-time
/// diagnostics with stable `RDOC###` codes where the contract defines one.
///
/// ```ignore (requires companion Component authoring files beside the invocation)
/// dioxus_registry_preview::component! {
///     group: "actions",
///     render_readme: true,
///     examples: {
///         overview { description: "Shows the normal Component state." },
///     },
/// }
/// ```
#[proc_macro]
pub fn component(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ComponentInvocation);

    expand_component(input)
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

/// Generates the Preview's catalog of Component pages from a Registry manifest.
///
/// The invocation requires `manifest`, `group`, `default`, and `catalog`; `repository` and
/// `revision` are optional. Without `repository`, the invoking package's Cargo repository is used.
/// The group block maps durable string IDs to Consumer-owned values. The catalog block names the
/// descriptor type, generated Component path prefix, rendering-kind constructor, and any
/// handwritten descriptors.
///
/// The macro emits `ComponentPage`, `PAGE_CATALOG`, `APP_CATALOG`, and a crate-visible
/// `example_modules` facade. `PAGE_CATALOG` contains handwritten descriptors followed by generated
/// Component pages in root Registry-manifest order. `APP_CATALOG` adds the stable group-ID mapping
/// and Registry installation facts consumed by `chrome::App`.
///
/// Every Registry member is validated from the invocation-relative root manifest. Unknown groups,
/// duplicate generated IDs, invalid defaults, and unsafe repository/revision values are reported
/// at compile time.
#[proc_macro]
pub fn component_pages(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as CatalogInvocation);

    expand_component_pages(input)
        .unwrap_or_else(catalog_failure)
        .into()
}

// Keeping one expansion function makes the generated contract inspectable in source order.
#[allow(clippy::too_many_lines)]
fn expand_component(input: ComponentInvocation) -> Result<proc_macro2::TokenStream> {
    let docs_crate = docs_crate()?;
    let call_file = proc_macro::Span::call_site()
        .local_file()
        .ok_or_else(|| syn::Error::new(Span::call_site(), "cannot locate the macro invocation"))?;
    let validation = load_component(&call_file, &input);

    if let Some(diagnostic) = validation.diagnostics.first() {
        return Err(component_error(diagnostic, &input));
    }
    let documentation = validation
        .value
        .expect("core returns documentation when validation succeeds");

    let component_root_value = input.component_root.value();
    let manifest_include = LitStr::new(
        &include_path(&component_root_value, "component.json"),
        Span::call_site(),
    );
    let source_include = LitStr::new(
        &include_path(&component_root_value, "component.rs"),
        Span::call_site(),
    );
    let readme_include = LitStr::new(
        &include_path(&component_root_value, "README.md"),
        Span::call_site(),
    );
    let readme_title = LitStr::new(&documentation.title, Span::call_site());
    let readme_html = LitStr::new(&documentation.readme_html, Span::call_site());
    let name = LitStr::new(&documentation.name, Span::call_site());
    let description = LitStr::new(&documentation.description, Span::call_site());
    let group_id = input.group;
    let page = Ident::new("DocumentationPage", Span::call_site());
    let example_section = input.example_section;
    let readme_section = input.readme_section;
    let listed = input.listed;
    let generated_readme_section = input.render_readme.then(|| {
        quote! {
            /// The README body rendered through the consumer's configured adapter.
            #[component]
            pub fn ReadmeSection() -> Element {
                use #readme_section as __RegistryPreviewReadmeSection;

                rsx! {
                    __RegistryPreviewReadmeSection { html: DOCUMENTATION.readme_html }
                }
            }
        }
    });
    let default_readme_call = input.render_readme.then(|| quote! { ReadmeSection {} });

    let mut module_declarations = Vec::new();
    let mut example_metadata = Vec::new();
    let mut generated_example_sections = Vec::new();
    let mut example_sections = Vec::new();

    for (index, (example, model)) in input
        .examples
        .into_iter()
        .zip(documentation.examples)
        .enumerate()
    {
        let module = example.module;
        let example_path = LitStr::new(&format!("examples/{module}.rs"), example.slug.span());
        let slug = LitStr::new(&model.slug, example.slug.span());
        let title = LitStr::new(
            &model.title,
            example
                .title
                .as_ref()
                .map_or_else(|| module.span(), LitStr::span),
        );
        let example_description = LitStr::new(&model.description, example.description.span());
        let index = syn::Index::from(index);
        let section = Ident::new(&example_section_name(&module.to_string()), module.span());
        let highlights = example_highlights(&docs_crate, &module, &model.source)?;

        module_declarations.push(quote! {
            #[path = #example_path]
            pub(crate) mod #module;
        });
        example_metadata.push(quote! {
            #docs_crate::ExampleDocumentation::new(
                #slug,
                #title,
                #example_description,
                include_str!(#example_path),
                #highlights
            )
        });
        generated_example_sections.push(quote! {
            /// One compiling Example rendered through the consumer's configured adapter.
            #[component]
            pub fn #section() -> Element {
                use #example_section as __RegistryPreviewExampleSection;

                rsx! {
                    __RegistryPreviewExampleSection {
                        documentation: DOCUMENTATION.examples[#index],
                        #module::Example {}
                    }
                }
            }
        });
        example_sections.push(quote! { #section {} });
    }

    let page_contents = input.page.map_or_else(
        || quote! { #default_readme_call #(#example_sections)* },
        |page| quote! { #page {} },
    );

    Ok(quote! {
        use ::dioxus::prelude::*;

        const _: &str = include_str!(#manifest_include);
        const _: &str = include_str!(#source_include);

        #(#module_declarations)*

        /// The stable group ID declared by this documentation module.
        pub const GROUP_ID: &'static str = #group_id;

        /// Whether the Preview lists this page in its navigation and index.
        pub const LISTED: bool = #listed;

        /// Documentation derived from this component's authoring files.
        pub const DOCUMENTATION: #docs_crate::ComponentDocumentation =
            #docs_crate::ComponentDocumentation::new(
                #name,
                #readme_title,
                #description,
                include_str!(#readme_include),
                #readme_html,
                &[
                    #(#example_metadata),*
                ],
            );

        #generated_readme_section
        #(#generated_example_sections)*

        /// The component's generated documentation page.
        #[component]
        pub fn #page() -> Element {
            rsx! {
                #page_contents
            }
        }
    })
}

// Keeping one expansion function makes the generated contract inspectable in source order.
#[allow(clippy::too_many_lines)]
fn expand_component_pages(input: CatalogInvocation) -> Result<proc_macro2::TokenStream> {
    let docs_crate = docs_crate()?;
    let call_file = proc_macro::Span::call_site()
        .local_file()
        .ok_or_else(|| syn::Error::new(Span::call_site(), "cannot locate the macro invocation"))?;
    let validation = load_catalog(&call_file, &input);

    if let Some(diagnostic) = validation.diagnostics.first() {
        return Err(catalog_error(diagnostic, &input));
    }
    let catalog = validation
        .value
        .expect("core returns a catalog when validation succeeds");
    let package_repository = std::env::var("CARGO_PKG_REPOSITORY")
        .ok()
        .filter(|repository| !repository.is_empty());
    let configured_repository = input.repository.as_ref().map(LitStr::value);
    let revision = input.revision.as_ref().map(LitStr::value);
    let source_validation = validate_registry_source(
        configured_repository
            .as_deref()
            .or(package_repository.as_deref()),
        revision.as_deref(),
    );

    if let Some(diagnostic) = source_validation.diagnostics.first() {
        return Err(catalog_error(diagnostic, &input));
    }

    let group_type = &input.group.ty;
    let entries = catalog
        .entries
        .iter()
        .map(|entry| {
            let module = Ident::new(&entry.module, input.manifest.span());
            let path = LitStr::new(
                &entry.docs_relative.to_string_lossy(),
                input.manifest.span(),
            );
            let group = input.group.mappings[entry.group_index].value.clone();
            (module, path, group, entry)
        })
        .collect::<Vec<_>>();
    let app_group_ids = entries
        .iter()
        .map(|(_, _, _, entry)| {
            let page_id = LitStr::new(&entry.id, input.catalog.path.span());
            let group_id = &input.group.mappings[entry.group_index].id;
            quote! { #page_id => #group_id }
        })
        .collect::<Vec<_>>();
    let app_group_values = input
        .group
        .mappings
        .iter()
        .map(|mapping| {
            let id = &mapping.id;
            let value = &mapping.value;
            quote! { #value => #id }
        })
        .collect::<Vec<_>>();
    let default_index = syn::Index::from(catalog.default_index);
    let module_declarations = entries.iter().map(|(module, path, _, _)| {
        quote! {
            #[path = #path]
            pub(crate) mod #module;
        }
    });
    let example_modules = entries.iter().map(|(module, _, _, _)| quote! { #module });
    let all = entries
        .iter()
        .enumerate()
        .map(|(index, _)| syn::Index::from(index))
        .map(|index| quote! { Self(#index) });
    let documentation = entries
        .iter()
        .enumerate()
        .map(|(index, (module, _, group, _))| {
            let index = syn::Index::from(index);
            quote! {
                #index => PageDocumentation::new(
                    #module::DOCUMENTATION,
                    #group,
                    #module::LISTED,
                )
            }
        });
    let views = entries
        .iter()
        .enumerate()
        .map(|(index, (module, _, _, _))| {
            let index = syn::Index::from(index);
            quote! {
                #index => rsx! {
                    #module::DocumentationPage {}
                }
            }
        });
    let descriptor_type = &input.catalog.descriptor;
    let component_kind = &input.catalog.component;
    let custom_pages = &input.catalog.custom;
    let path_prefix = input.catalog.path.value();
    let registry_description = LitStr::new(&catalog.registry_description, input.manifest.span());
    let registry_name = LitStr::new(&catalog.registry_name, input.manifest.span());
    let default_component = LitStr::new(&catalog.default_component, input.default.span());
    let component_names = catalog
        .entries
        .iter()
        .map(|entry| LitStr::new(&entry.id, input.manifest.span()))
        .collect::<Vec<_>>();
    let repository = input.repository.as_ref().map_or_else(
        || quote! { env!("CARGO_PKG_REPOSITORY") },
        |repository| quote! { #repository },
    );
    let registry_source = input.revision.as_ref().map_or_else(
        || quote! { #docs_crate::chrome::RegistrySource::git(#repository) },
        |revision| {
            quote! {
                #docs_crate::chrome::RegistrySource::git(#repository).with_revision(#revision)
            }
        },
    );
    let descriptors = entries
        .iter()
        .enumerate()
        .map(|(index, (_, _, group, entry))| {
            let index = syn::Index::from(index);
            let id = LitStr::new(&entry.id, input.catalog.path.span());
            let path = LitStr::new(
                &catalog_path(&path_prefix, &entry.id),
                input.catalog.path.span(),
            );
            let title = LitStr::new(&entry.title, input.catalog.path.span());
            let description = LitStr::new(&entry.description, input.catalog.path.span());
            let listing = if entry.listed {
                quote!(#docs_crate::ListingPolicy::Listed)
            } else {
                quote!(#docs_crate::ListingPolicy::Unlisted)
            };
            quote! {
                #docs_crate::PageDescriptor::new(
                    #id,
                    #path,
                    #title,
                    #description,
                    #docs_crate::NavigationPlacement::Group(#group),
                    #listing,
                    #docs_crate::BrowserTestPolicy::Enabled,
                    #component_kind(ComponentPage(#index)),
                )
            }
        });
    let manifest_include = input.manifest;

    Ok(quote! {
        use ::dioxus::prelude::*;

        const _: &str = include_str!(#manifest_include);

        #(#module_declarations)*

        /// Component documentation modules exposed for Examples that share fixtures.
        pub(crate) mod example_modules {
            #[allow(unused_imports)]
            pub(crate) use super::{#(#example_modules),*};
        }

        /// A generated Component page selected from the Registry catalog.
        #[derive(Copy, Clone, Debug, PartialEq, Eq)]
        pub struct ComponentPage(usize);

        impl ::std::default::Default for ComponentPage {
            fn default() -> Self {
                Self(#default_index)
            }
        }

        #[derive(Copy, Clone)]
        struct PageDocumentation {
            component: #docs_crate::ComponentDocumentation,
            group: #group_type,
            listed: bool,
        }

        /// Every handwritten and generated page in site catalog order.
        pub const PAGE_CATALOG: &'static [#descriptor_type] = &[
            #(#custom_pages,)*
            #(#descriptors),*
        ];

        #[doc(hidden)]
        #[allow(unreachable_patterns)]
        fn __registry_preview_group_id(page: &#descriptor_type) -> &'static str {
            match page.id {
                #(#app_group_ids),*,
                _ => match page.navigation {
                    #docs_crate::NavigationPlacement::Group(group) => match group {
                        #(#app_group_values),*,
                        _ => panic!("page catalog contains a Component group without a stable ID"),
                    },
                    #docs_crate::NavigationPlacement::Standalone(_) => {
                        unreachable!("standalone pages do not require a group ID")
                    },
                    _ => panic!("page catalog contains an unsupported navigation placement"),
                },
            }
        }

        /// The page catalog, stable group-ID mapping, and Registry facts
        /// consumed by `chrome::App`.
        pub const APP_CATALOG: #docs_crate::chrome::AppCatalog<#descriptor_type> =
            #docs_crate::chrome::AppCatalog::new(PAGE_CATALOG, __registry_preview_group_id)
                .with_description(#registry_description)
                .with_registry(#docs_crate::chrome::RegistryDocumentation::new(
                    #registry_name,
                    #registry_description,
                    &[#(#component_names),*],
                    #default_component,
                    #registry_source,
                ));

        impl PageDocumentation {
            const fn new(
                component: #docs_crate::ComponentDocumentation,
                group: #group_type,
                listed: bool,
            ) -> Self {
                Self { component, group, listed }
            }
        }

        impl ComponentPage {
            /// Every generated Component page in root Registry-manifest order.
            pub const ALL: &'static [Self] = &[#(#all),*];

            /// The Component's stable page and install name.
            #[must_use]
            pub const fn name(self) -> &'static str {
                self.documentation().component.name
            }

            /// The page title derived from the Component README.
            #[must_use]
            pub const fn title(self) -> &'static str {
                self.documentation().component.title
            }

            /// The catalog description from the Component manifest.
            #[must_use]
            pub const fn description(self) -> &'static str {
                self.documentation().component.description
            }

            /// The Consumer-owned group value resolved from the stable group ID.
            #[must_use]
            pub const fn group(self) -> #group_type {
                self.documentation().group
            }

            /// Whether navigation and indexes list this Component page.
            #[must_use]
            pub const fn listed(self) -> bool {
                self.documentation().listed
            }

            const fn documentation(self) -> PageDocumentation {
                match self.0 {
                    #(#documentation),*,
                    _ => unreachable!(),
                }
            }

            /// Renders the generated or Consumer-overridden Component page.
            ///
            /// # Errors
            ///
            /// Returns a Dioxus rendering error when the generated page cannot be rendered.
            pub fn view(self) -> ::dioxus::prelude::Element {
                match self.0 {
                    #(#views),*,
                    _ => unreachable!(),
                }
            }
        }

        impl ::std::fmt::Display for ComponentPage {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                formatter.write_str(self.name())
            }
        }

        /// An install name that does not identify a generated Component page.
        #[derive(Clone, Debug, PartialEq, Eq)]
        pub struct UnknownComponent(String);

        impl ::std::fmt::Display for UnknownComponent {
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                write!(formatter, "no page is for component '{}'", self.0)
            }
        }

        impl ::std::error::Error for UnknownComponent {}

        impl ::std::str::FromStr for ComponentPage {
            type Err = UnknownComponent;

            fn from_str(name: &str) -> ::std::result::Result<Self, Self::Err> {
                Self::ALL
                    .iter()
                    .copied()
                    .find(|page| page.name() == name)
                    .ok_or_else(|| UnknownComponent(name.to_owned()))
            }
        }
    })
}

fn component_error(diagnostic: &Diagnostic, input: &ComponentInvocation) -> syn::Error {
    let span = match diagnostic.target {
        DiagnosticTarget::ComponentRoot => input.component_root.span(),
        DiagnosticTarget::ComponentGroup => input.group.span(),
        DiagnosticTarget::ExampleSlug(index) => input
            .examples
            .get(index)
            .map_or_else(Span::call_site, |example| example.slug.span()),
        _ => Span::call_site(),
    };
    syn::Error::new(span, format!("{}: {}", diagnostic.code, diagnostic.message))
}

// A failed catalog still defines its page type so Consumer items that name it report the
// validation diagnostic alone instead of a cascade of unresolved-type errors.
fn catalog_failure(error: syn::Error) -> proc_macro2::TokenStream {
    let error = error.into_compile_error();
    quote! {
        #error

        /// Placeholder for the generated Component page type after failed catalog validation.
        #[allow(dead_code)]
        #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
        pub struct ComponentPage(usize);
    }
}

fn catalog_error(diagnostic: &Diagnostic, input: &CatalogInvocation) -> syn::Error {
    let span = match diagnostic.target {
        DiagnosticTarget::CatalogManifest => input.manifest.span(),
        DiagnosticTarget::CatalogGroup(index) => input
            .group
            .mappings
            .get(index)
            .map_or_else(Span::call_site, |mapping| mapping.id.span()),
        DiagnosticTarget::CatalogDefault => input.default.span(),
        DiagnosticTarget::CatalogRepository => input
            .repository
            .as_ref()
            .map_or_else(Span::call_site, LitStr::span),
        DiagnosticTarget::CatalogRevision => input
            .revision
            .as_ref()
            .map_or_else(Span::call_site, LitStr::span),
        _ => Span::call_site(),
    };
    syn::Error::new(span, format!("{}: {}", diagnostic.code, diagnostic.message))
}

fn include_path(component_root: &str, file: &str) -> String {
    PathBuf::from(component_root)
        .join(file)
        .to_string_lossy()
        .into_owned()
}

/// The highlight-span argument appended to `ExampleDocumentation::new`.
///
/// Without the `syntax-highlighting` feature the facade constructor has no such parameter and
/// the expansion stays free of tree-sitter.
#[cfg(feature = "syntax-highlighting")]
fn example_highlights(
    docs_crate: &proc_macro2::TokenStream,
    module: &Ident,
    source: &str,
) -> Result<proc_macro2::TokenStream> {
    highlight::rust_spans(docs_crate, source).map_err(|error| {
        syn::Error::new(
            module.span(),
            format!("cannot highlight the Example source for `{module}`: {error}"),
        )
    })
}

#[cfg(not(feature = "syntax-highlighting"))]
#[allow(clippy::unnecessary_wraps)]
fn example_highlights(
    _docs_crate: &proc_macro2::TokenStream,
    _module: &Ident,
    _source: &str,
) -> Result<proc_macro2::TokenStream> {
    Ok(proc_macro2::TokenStream::new())
}

fn catalog_path(prefix: &str, id: &str) -> String {
    let prefix = prefix.trim_end_matches('/');
    if prefix.is_empty() {
        format!("/{id}")
    } else {
        format!("{prefix}/{id}")
    }
}

fn docs_crate() -> Result<proc_macro2::TokenStream> {
    match crate_name("dioxus-registry-preview").map_err(|error| {
        syn::Error::new(
            Span::call_site(),
            format!("cannot resolve the dioxus-registry-preview dependency: {error}"),
        )
    })? {
        FoundCrate::Itself => Ok(quote!(crate)),
        FoundCrate::Name(name) => {
            let name = Ident::new(&name.replace('-', "_"), Span::call_site());
            Ok(quote!(::#name))
        }
    }
}
