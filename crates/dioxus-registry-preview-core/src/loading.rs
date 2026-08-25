use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use pulldown_cmark::{Event, Parser, Tag};
use serde::Deserialize;
use url::Url;

use crate::diagnostic::{Diagnostic, DiagnosticCode, DiagnosticTarget, Validation};
use crate::markdown::{MarkdownOptions, markdown_to_html};
use crate::syntax::{CatalogInvocation, ComponentInvocation, sentence_case, valid_group_id};

/// One stable Component group ID in site order.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DocumentationGroup {
    /// The durable lowercase kebab-case group ID.
    pub id: String,
}

/// The validated documentation authored for one Component.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ComponentDocumentation {
    /// The Component install/page name from its manifest.
    pub name: String,
    /// The page title from the README's H1.
    pub title: String,
    /// The catalog description from the Component manifest.
    pub description: String,
    /// The complete Component source file.
    pub source: String,
    /// The complete Component README.
    pub readme: String,
    /// The renderable README body converted to HTML.
    pub readme_html: String,
    /// The stable group ID authored by `component!`.
    pub group_id: String,
    /// Whether site navigation and indexes list the page.
    pub listed: bool,
    /// How the generated Component page is assembled.
    pub page: PageKind,
    /// Declared Examples in page order.
    pub examples: Vec<ExampleDocumentation>,
    /// The member path authored in the root Registry manifest.
    pub member_path: PathBuf,
    /// The path to the Component's `docs/mod.rs` invocation.
    pub docs_path: PathBuf,
    /// The resolved Component root directory.
    pub component_root: PathBuf,
}

/// The validated documentation authored for one declared, readable Example source file.
///
/// Core loading does not compile the source. Macro-generated modules enforce compilation later.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ExampleDocumentation {
    /// The Rust module identifier authored by `component!`.
    pub module: String,
    /// The URL fragment and browser-test identifier.
    pub slug: String,
    /// The rendered Example heading.
    pub title: String,
    /// What the Example demonstrates.
    pub description: String,
    /// The complete Example source file.
    pub source: String,
    /// The resolved Example source path.
    pub path: PathBuf,
}

/// How a Component page is assembled.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PageKind {
    /// Use the standard generated page assembly.
    Generated,
    /// Render the named Consumer-owned zero-prop Component.
    Custom(String),
}

/// One Component page in Registry manifest order.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DocumentationPage {
    /// The stable page ID.
    pub id: String,
    /// The referenced Component's index in [`DocumentationSite::components`].
    pub component_index: usize,
    /// The stable Component group ID.
    pub group_id: String,
    /// Whether navigation and indexes list the page.
    pub listed: bool,
    /// How the page is assembled.
    pub kind: PageKind,
}

/// One readable Component README, retained for consumer-owned validation rules.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct ComponentReadme {
    /// The member path authored in the root Registry manifest.
    pub member_path: PathBuf,
    /// The resolved README path.
    pub path: PathBuf,
    /// The complete README source.
    pub source: String,
}

/// The validated documentation model for an entire site.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct DocumentationSite {
    /// The name from the root Registry manifest.
    pub registry_name: String,
    /// The description from the root Registry manifest.
    pub registry_description: String,
    /// Stable Component groups in catalog order.
    pub groups: Vec<DocumentationGroup>,
    /// Validated Components in root-manifest order.
    pub components: Vec<ComponentDocumentation>,
    /// Component pages in root-manifest order.
    pub pages: Vec<DocumentationPage>,
    /// Readable Component READMEs retained for Consumer policy.
    pub readmes: Vec<ComponentReadme>,
    /// The default Component's Rust module name.
    pub default_component_module: String,
}

/// The subset of the site model needed by `component_pages!` code generation.
#[doc(hidden)]
#[derive(Debug)]
#[non_exhaustive]
pub struct CatalogDocumentation {
    /// The name from the root Registry manifest.
    pub registry_name: String,
    /// The description from the root Registry manifest.
    pub registry_description: String,
    /// Generated Component entries in root-manifest order.
    pub entries: Vec<CatalogEntry>,
    /// The default Component's index in `entries`.
    pub default_index: usize,
    /// The default Component's install name.
    pub default_component: String,
}

/// A validated Git source for installing Components from one Registry.
#[derive(Clone, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub struct RegistrySource {
    /// The absolute Git clone URL.
    pub repository: String,
    /// The optional pinned Git revision.
    pub revision: Option<String>,
}

/// One Registry member in a generated catalog.
#[doc(hidden)]
#[derive(Debug)]
#[non_exhaustive]
pub struct CatalogEntry {
    /// The member's Rust module name.
    pub module: String,
    /// The documentation module path relative to the catalog invocation.
    pub docs_relative: PathBuf,
    /// The member's index into the invocation's group mappings.
    pub group_index: usize,
    /// The stable Component group ID.
    pub group_id: String,
    /// The stable Component page/install ID.
    pub id: String,
    /// The page title.
    pub title: String,
    /// The page description.
    pub description: String,
    /// Whether navigation and indexes list the page.
    pub listed: bool,
}

#[derive(Deserialize)]
struct RegistryManifest {
    #[serde(default)]
    name: String,
    #[serde(default)]
    description: String,
    members: Vec<String>,
}

#[derive(Deserialize)]
struct ComponentManifest {
    name: String,
    description: String,
    #[serde(default)]
    exclude: Vec<String>,
}

struct RegistryMember {
    index: usize,
    declared: String,
    path: PathBuf,
    module: String,
    root: PathBuf,
    docs_path: PathBuf,
}

/// Loads and validates one complete documentation site from its `component_pages!` invocation.
///
/// Paths are resolved from `invocation_path`. The loader reads the root Registry manifest and
/// each member's manifest, source, README, documentation invocation, and declared Examples. It
/// aggregates independent diagnostics and may retain a partial site for additional Consumer
/// policy checks.
// The internal `expect` is guarded by member discovery's `Validation` invariant.
#[allow(clippy::missing_panics_doc, clippy::too_many_lines)]
pub fn load_site_from_catalog(
    invocation_path: &Path,
    markdown: &MarkdownOptions<'_>,
) -> Validation<DocumentationSite> {
    let source = match fs::read_to_string(invocation_path) {
        Ok(source) => source,
        Err(error) => {
            return Validation::failure(vec![
                Diagnostic::new(
                    DiagnosticCode::InvocationRead,
                    format!("cannot read catalog invocation: {error}"),
                )
                .at_path(invocation_path),
            ]);
        }
    };
    let input = match catalog_invocation(&source, invocation_path) {
        Ok(input) => input,
        Err(diagnostic) => return Validation::failure(vec![diagnostic]),
    };

    let mut diagnostics = validate_catalog_input(&input)
        .into_iter()
        .map(|diagnostic| diagnostic.at_path(invocation_path))
        .collect::<Vec<_>>();
    let invocation_root = invocation_path.parent().unwrap_or_else(|| Path::new(""));
    let relative_manifest = Path::new(&input.manifest.value()).to_owned();
    if relative_manifest.is_absolute() {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::InvalidMemberPath,
                "manifest must be relative to the macro invocation",
            )
            .at_path(invocation_path)
            .for_target(DiagnosticTarget::CatalogManifest),
        );
        return Validation::failure(diagnostics);
    }
    let registry_path = invocation_root.join(relative_manifest);
    let loaded_registry = load_registry_manifest(&registry_path, DiagnosticTarget::CatalogManifest);
    diagnostics.extend(loaded_registry.diagnostics);
    let Some(manifest) = loaded_registry.value else {
        return Validation::failure(diagnostics);
    };
    let discovered_members =
        discover_registry_members(&manifest, &registry_path, DiagnosticTarget::CatalogManifest);
    diagnostics.extend(discovered_members.diagnostics);
    let members = discovered_members
        .value
        .expect("member discovery always produces its valid subset");
    let groups = input
        .group
        .mappings
        .iter()
        .map(|mapping| DocumentationGroup {
            id: mapping.id.value(),
        })
        .collect::<Vec<_>>();
    let valid_groups = groups
        .iter()
        .map(|group| group.id.as_str())
        .collect::<HashSet<_>>();
    let default = input.default.to_string();
    let mut page_ids = HashMap::new();
    let mut components = Vec::new();
    let mut pages = Vec::new();
    let mut readmes = Vec::new();
    let mut default_exists = false;

    for member in members {
        let member_path = &member.path;
        let module = &member.module;
        default_exists |= module == &default;

        let member_root = member.root;
        validate_layout(&member_root, &mut diagnostics);
        let docs_path = member.docs_path;
        let invocation = match component_invocation_from_file(&docs_path) {
            Ok(invocation) => invocation,
            Err(diagnostic) => {
                diagnostics.push(diagnostic);
                validate_conventional_component(&member_root, &mut diagnostics);
                collect_readme(member_path, &member_root.join("README.md"), &mut readmes);
                continue;
            }
        };
        if let Some(component_root) = input_component_root(&invocation) {
            let readme_path = docs_path
                .parent()
                .unwrap_or_else(|| Path::new(""))
                .join(component_root)
                .join("README.md");
            collect_readme(member_path, &readme_path, &mut readmes);
        }
        let group_id = invocation.group.value();
        if !valid_groups.contains(group_id.as_str()) {
            let valid = groups
                .iter()
                .map(|group| format!("`{}`", group.id))
                .collect::<Vec<_>>()
                .join(", ");
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::UnknownGroup,
                    format!("unknown group ID `{group_id}`; valid group IDs: {valid}"),
                )
                .at_path(&docs_path),
            );
        }
        validate_custom_page(&docs_path, &invocation, &mut diagnostics);
        validate_required_excludes(&docs_path, &invocation, &mut diagnostics);

        let loaded = load_component_with_markdown(&docs_path, &invocation, markdown);
        diagnostics.extend(loaded.diagnostics);
        if let Some(mut component) = loaded.value {
            member_path.clone_into(&mut component.member_path);
            let component_index = components.len();
            if let Some(previous) = page_ids.insert(component.name.clone(), member.declared.clone())
            {
                diagnostics.push(
                    Diagnostic::new(
                        DiagnosticCode::DuplicatePageId,
                        format!(
                            "duplicate Component page ID `{}` in `{previous}` and `{}`",
                            component.name, member.declared
                        ),
                    )
                    .at_path(&registry_path),
                );
            }
            pages.push(DocumentationPage {
                id: component.name.clone(),
                component_index,
                group_id: component.group_id.clone(),
                listed: component.listed,
                kind: component.page.clone(),
            });
            components.push(component);
        }
    }

    if !default_exists {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::InvalidDefault,
                format!("default Component `{default}` is not a Registry member"),
            )
            .at_path(invocation_path)
            .for_target(DiagnosticTarget::CatalogDefault),
        );
    }

    let site = DocumentationSite {
        registry_name: manifest.name,
        registry_description: manifest.description,
        groups,
        components,
        pages,
        readmes,
        default_component_module: default,
    };
    Validation {
        value: Some(site),
        diagnostics,
    }
}

fn catalog_invocation(source: &str, path: &Path) -> Result<CatalogInvocation, Diagnostic> {
    let file = syn::parse_file(source).map_err(|error| {
        Diagnostic::new(
            DiagnosticCode::InvocationParse,
            format!("cannot parse {}: {error}", path.display()),
        )
        .at_path(path)
    })?;
    let tokens = catalog_invocation_tokens(file.items);
    let tokens = tokens.ok_or_else(|| {
        Diagnostic::new(
            DiagnosticCode::MissingInvocation,
            format!("{} contains no component_pages! invocation", path.display()),
        )
        .at_path(path)
    })?;
    syn::parse2(tokens).map_err(|error| {
        Diagnostic::new(
            DiagnosticCode::InvocationParse,
            format!(
                "cannot parse the component_pages! invocation at {}: {error}",
                path.display()
            ),
        )
        .at_path(path)
    })
}

fn catalog_invocation_tokens(items: Vec<syn::Item>) -> Option<proc_macro2::TokenStream> {
    for item in items {
        match item {
            syn::Item::Macro(item)
                if item
                    .mac
                    .path
                    .segments
                    .last()
                    .is_some_and(|segment| segment.ident == "component_pages") =>
            {
                return Some(item.mac.tokens);
            }
            syn::Item::Mod(item) => {
                let Some((_, items)) = item.content else {
                    continue;
                };
                if let Some(tokens) = catalog_invocation_tokens(items) {
                    return Some(tokens);
                }
            }
            _ => {}
        }
    }
    None
}

/// Loads one Component's model with the documented default Markdown behavior.
///
/// Unlike full-site loading, any diagnostic prevents this function from returning a value.
pub fn load_component(
    invocation_path: &Path,
    input: &ComponentInvocation,
) -> Validation<ComponentDocumentation> {
    load_component_with_markdown(invocation_path, input, &MarkdownOptions::default())
}

/// Loads one Component's model with caller-selected Markdown behavior.
///
/// Authoring paths are resolved from `invocation_path`. Any diagnostic prevents this function
/// from returning a value. Markdown and hook output follow the trust requirements documented by
/// [`crate::markdown_to_html`].
// Internal `expect` calls are guarded by the diagnostics-empty check.
#[allow(clippy::missing_panics_doc)]
pub fn load_component_with_markdown(
    invocation_path: &Path,
    input: &ComponentInvocation,
    markdown: &MarkdownOptions<'_>,
) -> Validation<ComponentDocumentation> {
    let mut diagnostics = validate_component_input(input);
    let docs_root = invocation_path.parent().unwrap_or_else(|| Path::new(""));
    let relative_root = Path::new(&input.component_root.value()).to_owned();
    if relative_root.is_absolute() {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::InvalidMemberPath,
                "component_root must be relative to docs/mod.rs",
            )
            .for_target(DiagnosticTarget::ComponentRoot),
        );
        return Validation::failure(diagnostics);
    }
    let component_root = docs_root.join(&relative_root);
    let manifest_path = component_root.join("component.json");
    let source_path = component_root.join("component.rs");
    let readme_path = component_root.join("README.md");

    let manifest = read_component_manifest(&manifest_path, &mut diagnostics);
    let source = read_component_source(&source_path, &mut diagnostics);
    let readme = read_component_readme(&readme_path, &mut diagnostics);
    let mut examples = Vec::new();
    for (index, example) in input.examples.iter().enumerate() {
        let path = docs_root
            .join("examples")
            .join(format!("{}.rs", example.module));
        match fs::read_to_string(&path) {
            Ok(source) => examples.push(ExampleDocumentation {
                module: example.module.to_string(),
                slug: example.slug.value(),
                title: example.title.as_ref().map_or_else(
                    || sentence_case(&example.module.to_string()),
                    syn::LitStr::value,
                ),
                description: example.description.value(),
                source,
                path,
            }),
            Err(error) => diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::ExampleRead,
                    format!("cannot read {}: {error}", path.display()),
                )
                .at_path(&path)
                .for_target(DiagnosticTarget::ExampleSlug(index)),
            ),
        }
    }

    let has_readme_parts = readme.as_deref().and_then(readme_parts).is_some();
    if readme.is_some() && !has_readme_parts {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::MalformedReadme,
                format!(
                    "{} must start with an H1 and an introductory paragraph",
                    readme_path.display()
                ),
            )
            .at_path(&readme_path)
            .for_target(DiagnosticTarget::ComponentRoot),
        );
    }

    if !diagnostics.is_empty() {
        return Validation::failure(diagnostics);
    }

    let manifest = manifest.expect("a successful validation has a manifest");
    let readme = readme.expect("a successful validation has a README");
    let (title, body) = readme_parts(&readme).expect("a successful validation has README parts");
    let title = title.to_owned();
    let readme_html = markdown_to_html(body, markdown);
    let source = source.expect("a successful validation has component source");
    Validation::success(ComponentDocumentation {
        name: manifest.name,
        title,
        description: manifest.description,
        source,
        readme,
        readme_html,
        group_id: input.group.value(),
        listed: input.listed,
        page: input.page.as_ref().map_or(PageKind::Generated, |page| {
            PageKind::Custom(page.to_string())
        }),
        examples,
        member_path: PathBuf::new(),
        docs_path: invocation_path.to_owned(),
        component_root,
    })
}

/// Finds and parses the first top-level `component!` invocation in a module.
///
/// # Errors
///
/// Returns a diagnostic when `source` is malformed Rust, contains no top-level `component!`
/// invocation, or contains an invocation that does not match the supported grammar. `path` is
/// attached to that diagnostic but is not read by this function.
pub fn component_invocation(source: &str, path: &Path) -> Result<ComponentInvocation, Diagnostic> {
    let file = syn::parse_file(source).map_err(|error| {
        Diagnostic::new(
            DiagnosticCode::InvocationParse,
            format!("cannot parse {}: {error}", path.display()),
        )
        .at_path(path)
    })?;
    let tokens = file.items.into_iter().find_map(|item| {
        let syn::Item::Macro(item) = item else {
            return None;
        };
        item.mac
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "component")
            .then_some(item.mac.tokens)
    });
    let tokens = tokens.ok_or_else(|| {
        Diagnostic::new(
            DiagnosticCode::MissingInvocation,
            format!("{} contains no component! invocation", path.display()),
        )
        .at_path(path)
    })?;
    syn::parse2(tokens).map_err(|error| {
        Diagnostic::new(
            DiagnosticCode::InvocationParse,
            format!(
                "cannot parse the component! invocation at {}: {error}",
                path.display()
            ),
        )
        .at_path(path)
    })
}

/// Loads the Registry catalog used by `component_pages!` without Dioxus code generation.
///
/// This code-generation seam returns no value when any diagnostic is present. Consumer validation
/// should normally use [`load_site_from_catalog`] through the facade instead.
// The internal `expect` is guarded by member discovery's `Validation` invariant.
#[allow(clippy::missing_panics_doc, clippy::too_many_lines)]
pub fn load_catalog(
    invocation_path: &Path,
    input: &CatalogInvocation,
) -> Validation<CatalogDocumentation> {
    let mut diagnostics = validate_catalog_input(input);
    let invocation_root = invocation_path.parent().unwrap_or_else(|| Path::new(""));
    let relative_manifest = Path::new(&input.manifest.value()).to_owned();
    if relative_manifest.is_absolute() {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::InvalidMemberPath,
                "manifest must be relative to the macro invocation",
            )
            .for_target(DiagnosticTarget::CatalogManifest),
        );
        return Validation::failure(diagnostics);
    }
    let manifest_path = invocation_root.join(&relative_manifest);
    let loaded_registry = load_registry_manifest(&manifest_path, DiagnosticTarget::CatalogManifest);
    diagnostics.extend(loaded_registry.diagnostics);
    let Some(manifest) = loaded_registry.value else {
        return Validation::failure(diagnostics);
    };
    let discovered_members =
        discover_registry_members(&manifest, &manifest_path, DiagnosticTarget::CatalogManifest);
    diagnostics.extend(discovered_members.diagnostics);
    let members = discovered_members
        .value
        .expect("member discovery always produces its valid subset");
    let relative_registry_root = relative_manifest.parent().unwrap_or_else(|| Path::new(""));
    let mut entries = Vec::new();
    let mut default_index = None;
    for member in members {
        if input.default == member.module {
            default_index = Some(member.index);
        }

        let docs_path = member.docs_path;
        let invocation = match component_invocation_from_file(&docs_path) {
            Ok(invocation) => invocation,
            Err(diagnostic) => {
                diagnostics.push(diagnostic.for_target(DiagnosticTarget::CatalogManifest));
                continue;
            }
        };
        let group_id = invocation.group.value();
        let group_index = input
            .group
            .mappings
            .iter()
            .position(|mapping| mapping.id.value() == group_id);
        let Some(group_index) = group_index else {
            let valid = input
                .group
                .mappings
                .iter()
                .map(|mapping| format!("`{}`", mapping.id.value()))
                .collect::<Vec<_>>()
                .join(", ");
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::UnknownGroup,
                    format!(
                        "unknown group ID `{group_id}` in the component! invocation at {}; valid group IDs: {valid}",
                        docs_path.display()
                    ),
                )
                .for_target(DiagnosticTarget::CatalogManifest),
            );
            continue;
        };
        let loaded_component = load_component(&docs_path, &invocation);
        diagnostics.extend(loaded_component.diagnostics);
        let Some(component) = loaded_component.value else {
            continue;
        };
        entries.push(CatalogEntry {
            module: member.module,
            docs_relative: relative_registry_root
                .join(&member.path)
                .join("docs/mod.rs"),
            group_index,
            group_id,
            id: component.name,
            title: component.title,
            description: component.description,
            listed: component.listed,
        });
    }

    let default_index = if let Some(index) = default_index {
        index
    } else {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::InvalidDefault,
                format!(
                    "default Component `{}` is not a Registry member",
                    input.default
                ),
            )
            .for_target(DiagnosticTarget::CatalogDefault),
        );
        0
    };
    if diagnostics.is_empty() {
        let default_component = entries[default_index].id.clone();
        Validation::success(CatalogDocumentation {
            registry_name: manifest.name,
            registry_description: manifest.description,
            entries,
            default_index,
            default_component,
        })
    } else {
        Validation::failure(diagnostics)
    }
}

/// Validates the Git source used to generate Registry installation instructions.
///
/// Accepted repositories are absolute `git`, `http`, `https`, or `ssh` URLs. Revisions must be
/// nonempty, contain no whitespace, and not begin with `-` so generated commands remain safe.
// The internal `expect` is guarded by repository validation and the diagnostics-empty check.
#[allow(clippy::missing_panics_doc)]
pub fn validate_registry_source(
    repository: Option<&str>,
    revision: Option<&str>,
) -> Validation<RegistrySource> {
    let mut diagnostics = Vec::new();
    match repository {
        Some(repository) => validate_registry_repository(repository, &mut diagnostics),
        None => diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::MissingRegistryRepository,
                "no Registry repository is configured; set package.repository in Cargo.toml or add repository to component_pages!",
            )
            .for_target(DiagnosticTarget::CatalogRepository),
        ),
    }
    if let Some(revision) = revision {
        validate_registry_revision(revision, &mut diagnostics);
    }

    if diagnostics.is_empty() {
        Validation::success(RegistrySource {
            repository: repository
                .expect("a successful validation has a repository")
                .to_owned(),
            revision: revision.map(str::to_owned),
        })
    } else {
        Validation::failure(diagnostics)
    }
}

fn load_registry_manifest(path: &Path, target: DiagnosticTarget) -> Validation<RegistryManifest> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            return Validation::failure(vec![
                Diagnostic::new(
                    DiagnosticCode::RegistryManifestRead,
                    format!("cannot read {}: {error}", path.display()),
                )
                .at_path(path)
                .for_target(target),
            ]);
        }
    };
    let manifest = match serde_json::from_str::<RegistryManifest>(&source) {
        Ok(manifest) => manifest,
        Err(error) => {
            return Validation::failure(vec![
                Diagnostic::new(
                    DiagnosticCode::RegistryManifestParse,
                    format!("cannot parse {}: {error}", path.display()),
                )
                .at_path(path)
                .for_target(target),
            ]);
        }
    };
    let mut diagnostics = Vec::new();
    if manifest.name.trim().is_empty() {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::InvalidRegistryName,
                "the Registry manifest name must be nonempty",
            )
            .at_path(path)
            .for_target(target),
        );
    }
    if manifest.members.is_empty() {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::EmptyRegistry,
                "the Registry manifest lists no members",
            )
            .at_path(path)
            .for_target(target),
        );
    }
    Validation {
        value: Some(manifest),
        diagnostics,
    }
}

fn discover_registry_members(
    manifest: &RegistryManifest,
    manifest_path: &Path,
    target: DiagnosticTarget,
) -> Validation<Vec<RegistryMember>> {
    let registry_root = manifest_path.parent().unwrap_or_else(|| Path::new(""));
    let mut diagnostics = Vec::new();
    let mut modules = HashSet::new();
    let mut members = Vec::new();
    for (index, declared) in manifest.members.iter().enumerate() {
        let path = Path::new(declared);
        if !normalized_relative_path(declared, path) {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::InvalidMemberPath,
                    format!("Registry member `{declared}` must be a normalized relative path"),
                )
                .at_path(manifest_path)
                .for_target(target),
            );
            continue;
        }
        let Some(module) = path.file_name().and_then(|name| name.to_str()) else {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::InvalidMemberPath,
                    format!("Registry member `{declared}` has no UTF-8 directory name"),
                )
                .at_path(manifest_path)
                .for_target(target),
            );
            continue;
        };
        if module.starts_with("r#") || syn::parse_str::<syn::Ident>(module).is_err() {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::InvalidMemberPath,
                    format!("Registry member `{declared}` does not end in a Rust module name"),
                )
                .at_path(manifest_path)
                .for_target(target),
            );
            continue;
        }
        if !modules.insert(module.to_owned()) {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::DuplicateMemberModule,
                    format!("duplicate Registry member module `{module}`"),
                )
                .at_path(manifest_path)
                .for_target(target),
            );
        }
        let root = registry_root.join(path);
        members.push(RegistryMember {
            index,
            declared: declared.clone(),
            path: path.to_owned(),
            module: module.to_owned(),
            docs_path: root.join("docs/mod.rs"),
            root,
        });
    }
    Validation {
        value: Some(members),
        diagnostics,
    }
}

/// Separates a README's page title and catalog lead from its renderable body.
///
/// The README must begin with an H1 and an introductory Markdown paragraph. Both LF and CRLF line
/// endings are accepted, and the introduction may end at end of input. The returned body excludes
/// the heading and introduction.
#[must_use]
pub fn readme_parts(readme: &str) -> Option<(&str, &str)> {
    let heading_end = readme.find('\n')?;
    let title = readme[..heading_end].strip_prefix("# ")?.trim();
    if title.is_empty() {
        return None;
    }
    let after_heading = readme[heading_end + 1..].trim_start_matches(['\r', '\n']);
    let introduction_end = ["\n\n", "\r\n\r\n"]
        .into_iter()
        .filter_map(|separator| after_heading.find(separator))
        .min()
        .unwrap_or(after_heading.len());
    let introduction = after_heading[..introduction_end].trim();
    if !matches!(
        Parser::new(introduction).next(),
        Some(Event::Start(Tag::Paragraph))
    ) {
        return None;
    }
    Some((
        title,
        after_heading[introduction_end..].trim_start_matches(['\r', '\n']),
    ))
}

fn validate_component_input(input: &ComponentInvocation) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if !valid_group_id(&input.group.value()) {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::InvalidGroupId,
                "group ID must be nonempty lowercase kebab-case",
            )
            .for_target(DiagnosticTarget::ComponentGroup),
        );
    }
    if input.examples.is_empty() {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::EmptyExamples,
            "a component page must declare at least one example",
        ));
    }
    let mut slugs = HashSet::new();
    for (index, example) in input.examples.iter().enumerate() {
        let slug = example.slug.value();
        if slug.is_empty() {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::EmptyExampleSlug,
                    "example slug cannot be empty",
                )
                .for_target(DiagnosticTarget::ExampleSlug(index)),
            );
        } else if !slugs.insert(slug) {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::DuplicateExampleSlug,
                    "duplicate example slug",
                )
                .for_target(DiagnosticTarget::ExampleSlug(index)),
            );
        }
    }
    diagnostics
}

fn validate_catalog_input(input: &CatalogInvocation) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if let Some(repository) = &input.repository {
        validate_registry_repository(&repository.value(), &mut diagnostics);
    }
    if let Some(revision) = &input.revision {
        validate_registry_revision(&revision.value(), &mut diagnostics);
    }
    if input.group.mappings.is_empty() {
        diagnostics.push(Diagnostic::new(
            DiagnosticCode::EmptyGroupMapping,
            "group mapping must declare at least one ID",
        ));
    }
    let mut seen = HashSet::new();
    for (index, mapping) in input.group.mappings.iter().enumerate() {
        let id = mapping.id.value();
        if !valid_group_id(&id) {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::InvalidGroupId,
                    "group ID must be nonempty lowercase kebab-case",
                )
                .for_target(DiagnosticTarget::CatalogGroup(index)),
            );
        }
        if !seen.insert(id) {
            diagnostics.push(
                Diagnostic::new(DiagnosticCode::DuplicateGroupId, "duplicate group ID")
                    .for_target(DiagnosticTarget::CatalogGroup(index)),
            );
        }
    }
    diagnostics
}

fn validate_registry_repository(repository: &str, diagnostics: &mut Vec<Diagnostic>) {
    let valid = repository.trim() == repository
        && !repository
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
        && Url::parse(repository).is_ok_and(|url| {
            matches!(url.scheme(), "git" | "http" | "https" | "ssh") && url.host().is_some()
        });
    if !valid {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::InvalidRegistryRepository,
                "Registry repository must be an absolute git, http, https, or ssh URL",
            )
            .for_target(DiagnosticTarget::CatalogRepository),
        );
    }
}

fn validate_registry_revision(revision: &str, diagnostics: &mut Vec<Diagnostic>) {
    if revision.is_empty()
        || revision.starts_with('-')
        || revision
            .chars()
            .any(|character| character.is_control() || character.is_whitespace())
    {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::InvalidRegistryRevision,
                "Registry revision must be a nonempty Git revision without whitespace and may not start with `-`",
            )
            .for_target(DiagnosticTarget::CatalogRevision),
        );
    }
}

fn read_component_manifest(
    path: &Path,
    diagnostics: &mut Vec<Diagnostic>,
) -> Option<ComponentManifest> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::ComponentManifestRead,
                    format!("cannot read {}: {error}", path.display()),
                )
                .at_path(path)
                .for_target(DiagnosticTarget::ComponentRoot),
            );
            return None;
        }
    };
    match serde_json::from_str::<ComponentManifest>(&source) {
        Ok(manifest) => {
            if !valid_page_id(&manifest.name) {
                diagnostics.push(
                    Diagnostic::new(
                        DiagnosticCode::InvalidComponentName,
                        format!(
                            "Component name `{}` must be a nonempty lowercase slug",
                            manifest.name
                        ),
                    )
                    .at_path(path)
                    .for_target(DiagnosticTarget::ComponentRoot),
                );
            }
            Some(manifest)
        }
        Err(error) => {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::ComponentManifestParse,
                    format!("cannot parse {}: {error}", path.display()),
                )
                .at_path(path)
                .for_target(DiagnosticTarget::ComponentRoot),
            );
            None
        }
    }
}

fn read_component_source(path: &Path, diagnostics: &mut Vec<Diagnostic>) -> Option<String> {
    let source = match fs::read_to_string(path) {
        Ok(source) => source,
        Err(error) => {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::ComponentSourceRead,
                    format!("cannot read {}: {error}", path.display()),
                )
                .at_path(path)
                .for_target(DiagnosticTarget::ComponentRoot),
            );
            return None;
        }
    };
    let file = match syn::parse_file(&source) {
        Ok(file) => file,
        Err(error) => {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::ComponentSourceParse,
                    format!("cannot parse {}: {error}", path.display()),
                )
                .at_path(path)
                .for_target(DiagnosticTarget::ComponentRoot),
            );
            return None;
        }
    };
    if !file.items.iter().any(is_public_component) {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::MissingPublicComponent,
                format!(
                    "{} declares no public #[component] function",
                    path.display()
                ),
            )
            .at_path(path)
            .for_target(DiagnosticTarget::ComponentRoot),
        );
    }
    Some(source)
}

fn read_component_readme(path: &Path, diagnostics: &mut Vec<Diagnostic>) -> Option<String> {
    match fs::read_to_string(path) {
        Ok(readme) => Some(readme),
        Err(error) => {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::ReadmeRead,
                    format!("cannot read {}: {error}", path.display()),
                )
                .at_path(path)
                .for_target(DiagnosticTarget::ComponentRoot),
            );
            None
        }
    }
}

fn component_invocation_from_file(path: &Path) -> Result<ComponentInvocation, Diagnostic> {
    let source = fs::read_to_string(path).map_err(|error| {
        Diagnostic::new(
            if path.exists() {
                DiagnosticCode::InvocationRead
            } else {
                DiagnosticCode::MissingDocsModule
            },
            if path.exists() {
                format!("cannot read {}: {error}", path.display())
            } else {
                "Component docs has no mod.rs".to_owned()
            },
        )
        .at_path(path)
    })?;
    component_invocation(&source, path)
}

fn validate_layout(member_root: &Path, diagnostics: &mut Vec<Diagnostic>) {
    for obsolete in [member_root.join("docs.md"), member_root.join("examples")] {
        if obsolete.exists() {
            diagnostics.push(
                Diagnostic::new(
                    DiagnosticCode::SupersededLayout,
                    "Component still uses the superseded docs.md/examples layout",
                )
                .at_path(obsolete),
            );
        }
    }
    let examples = member_root.join("docs/examples");
    if !examples.is_dir() {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::MissingExamplesDirectory,
                "Component docs has no examples directory",
            )
            .at_path(examples),
        );
    }
}

fn validate_conventional_component(member_root: &Path, diagnostics: &mut Vec<Diagnostic>) {
    let manifest_path = member_root.join("component.json");
    let source_path = member_root.join("component.rs");
    let readme_path = member_root.join("README.md");
    read_component_manifest(&manifest_path, diagnostics);
    read_component_source(&source_path, diagnostics);
    let readme = read_component_readme(&readme_path, diagnostics);
    if readme
        .as_deref()
        .is_some_and(|readme| readme_parts(readme).is_none())
    {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::MalformedReadme,
                format!(
                    "{} must start with an H1 and an introductory paragraph",
                    readme_path.display()
                ),
            )
            .at_path(&readme_path),
        );
    }
    validate_required_excludes_at_path(&manifest_path, diagnostics);
}

fn input_component_root(input: &ComponentInvocation) -> Option<PathBuf> {
    let root = PathBuf::from(input.component_root.value());
    (!root.is_absolute()).then_some(root)
}

fn collect_readme(member_path: &Path, path: &Path, readmes: &mut Vec<ComponentReadme>) {
    if let Ok(source) = fs::read_to_string(path) {
        readmes.push(ComponentReadme {
            member_path: member_path.to_owned(),
            path: path.to_owned(),
            source,
        });
    }
}

fn validate_required_excludes(
    docs_path: &Path,
    input: &ComponentInvocation,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let docs_root = docs_path.parent().unwrap_or_else(|| Path::new(""));
    let Some(component_root) = input_component_root(input) else {
        return;
    };
    let manifest_path = docs_root.join(component_root).join("component.json");
    validate_required_excludes_at_path(&manifest_path, diagnostics);
}

fn validate_required_excludes_at_path(manifest_path: &Path, diagnostics: &mut Vec<Diagnostic>) {
    let Ok(source) = fs::read_to_string(manifest_path) else {
        return;
    };
    let Ok(manifest) = serde_json::from_str::<ComponentManifest>(&source) else {
        return;
    };
    let excludes = manifest
        .exclude
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let missing = ["component.json", "README.md", "docs"]
        .into_iter()
        .filter(|required| !excludes.contains(required))
        .collect::<Vec<_>>();
    if !missing.is_empty() {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::MissingExclude,
                format!("Component manifest does not exclude: {}", missing.join(",")),
            )
            .at_path(manifest_path),
        );
    }
}

fn validate_custom_page(
    docs_path: &Path,
    input: &ComponentInvocation,
    diagnostics: &mut Vec<Diagnostic>,
) {
    let Some(page) = &input.page else {
        return;
    };
    let Ok(source) = fs::read_to_string(docs_path) else {
        return;
    };
    let Ok(file) = syn::parse_file(&source) else {
        return;
    };
    let valid = file.items.iter().any(|item| {
        let syn::Item::Fn(function) = item else {
            return false;
        };
        function.sig.ident == *page
            && function.sig.inputs.is_empty()
            && function
                .attrs
                .iter()
                .any(|attribute| attribute.path().is_ident("component"))
    });
    if !valid {
        diagnostics.push(
            Diagnostic::new(
                DiagnosticCode::InvalidPage,
                format!("page `{page}` must name a zero-prop #[component] function in this module"),
            )
            .at_path(docs_path),
        );
    }
}

fn normalized_relative_path(declared: &str, path: &Path) -> bool {
    !declared.contains('\\')
        && !path.is_absolute()
        && path
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
}

fn valid_page_id(value: &str) -> bool {
    !value.is_empty()
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
        && !value.starts_with(['-', '_'])
        && !value.ends_with(['-', '_'])
}

fn is_public_component(item: &syn::Item) -> bool {
    let syn::Item::Fn(function) = item else {
        return false;
    };
    matches!(function.vis, syn::Visibility::Public(_))
        && function
            .attrs
            .iter()
            .any(|attribute| attribute.path().is_ident("component"))
}
