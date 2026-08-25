use proc_macro2::Span;
use syn::parse::{Parse, ParseStream};
use syn::{Expr, ExprPath, Ident, LitBool, LitStr, Path, Result, Token, Type, braced, bracketed};

/// The authoring input to one `component!` invocation.
#[derive(Debug)]
#[non_exhaustive]
pub struct ComponentInvocation {
    /// The Component root relative to the invocation's `docs/mod.rs` file.
    pub component_root: LitStr,
    /// The Component's stable group ID.
    pub group: LitStr,
    /// The Consumer adapter used to render one Example section.
    pub example_section: Path,
    /// The Consumer adapter used to render the README body.
    pub readme_section: Path,
    /// Whether the default page renders the README body.
    pub render_readme: bool,
    /// Whether navigation and indexes list the Component page.
    pub listed: bool,
    /// An optional Consumer-owned page Component.
    pub page: Option<Ident>,
    /// Declared Examples in page order.
    pub examples: Vec<ExampleInvocation>,
}

/// One Example declared by `component!`.
#[derive(Debug)]
#[non_exhaustive]
pub struct ExampleInvocation {
    /// The non-raw Rust module identifier.
    pub module: Ident,
    /// The URL fragment and browser-test identifier.
    pub slug: LitStr,
    /// An optional rendered heading override.
    pub title: Option<LitStr>,
    /// What the Example demonstrates.
    pub description: LitStr,
}

/// The authoring input to a `component_pages!` invocation.
#[derive(Debug)]
#[non_exhaustive]
pub struct CatalogInvocation {
    /// The root Registry manifest relative to the invocation file.
    pub manifest: LitStr,
    /// An optional Registry repository override.
    pub repository: Option<LitStr>,
    /// An optional pinned Git revision.
    pub revision: Option<LitStr>,
    /// The complete stable group-ID mapping.
    pub group: GroupInvocation,
    /// The default Component's Rust module name.
    pub default: Ident,
    /// The generated page catalog configuration.
    pub catalog: PageCatalogInvocation,
}

/// How `component_pages!` emits the consuming site's complete page catalog.
#[derive(Debug)]
#[non_exhaustive]
pub struct PageCatalogInvocation {
    /// The Consumer's page descriptor type.
    pub descriptor: Type,
    /// The absolute path prefix for generated Component pages.
    pub path: LitStr,
    /// The Consumer's generated-Component rendering-kind constructor.
    pub component: ExprPath,
    /// Handwritten descriptors placed before generated Component pages.
    pub custom: Vec<Expr>,
}

/// The consuming site's stable group-ID mapping.
#[derive(Debug)]
#[non_exhaustive]
pub struct GroupInvocation {
    /// The Consumer's group value type.
    pub ty: Path,
    /// Stable IDs and corresponding Consumer values in presentation order.
    pub mappings: Vec<GroupMapping>,
}

/// One stable group ID and the consuming site's value for it.
#[derive(Debug)]
#[non_exhaustive]
pub struct GroupMapping {
    /// The durable lowercase kebab-case ID.
    pub id: LitStr,
    /// The Consumer-owned group value.
    pub value: Path,
}

impl Parse for ComponentInvocation {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut component_root = None;
        let mut group = None;
        let mut example_section = None;
        let mut readme_section = None;
        let mut render_readme = None;
        let mut listed = None;
        let mut page = None;
        let mut examples = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "component_root" => set_once(&mut component_root, input.parse()?, &key)?,
                "group" => set_once(&mut group, input.parse()?, &key)?,
                "example_section" => set_once(&mut example_section, input.parse()?, &key)?,
                "readme_section" => set_once(&mut readme_section, input.parse()?, &key)?,
                "render_readme" => set_once(&mut render_readme, input.parse::<LitBool>()?, &key)?,
                "listed" => set_once(&mut listed, input.parse::<LitBool>()?, &key)?,
                "page" => set_once(&mut page, input.parse()?, &key)?,
                "examples" => {
                    let content;
                    braced!(content in input);
                    set_once(&mut examples, parse_examples(&content)?, &key)?;
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        "expected component_root, group, example_section, readme_section, render_readme, listed, page, or examples",
                    ));
                }
            }

            parse_comma(input)?;
        }

        Ok(Self {
            component_root: component_root.unwrap_or_else(|| LitStr::new("..", Span::call_site())),
            group: required(group, input, "group")?,
            example_section: example_section
                .unwrap_or_else(|| syn::parse_quote!(crate::example::ExampleSection)),
            readme_section: readme_section
                .unwrap_or_else(|| syn::parse_quote!(crate::example::ReadmeSection)),
            render_readme: render_readme.is_some_and(|value| value.value),
            listed: listed.is_none_or(|value| value.value),
            page,
            examples: required(examples, input, "examples")?,
        })
    }
}

impl Parse for CatalogInvocation {
    fn parse(input: ParseStream<'_>) -> Result<Self> {
        let mut manifest = None;
        let mut repository = None;
        let mut revision = None;
        let mut group = None;
        let mut default = None;
        let mut catalog = None;

        while !input.is_empty() {
            let key: Ident = input.parse()?;
            input.parse::<Token![:]>()?;

            match key.to_string().as_str() {
                "manifest" => set_once(&mut manifest, input.parse()?, &key)?,
                "repository" => set_once(&mut repository, input.parse()?, &key)?,
                "revision" => set_once(&mut revision, input.parse()?, &key)?,
                "group" => set_once(&mut group, parse_group(input)?, &key)?,
                "default" => set_once(&mut default, input.parse()?, &key)?,
                "catalog" => set_once(&mut catalog, parse_page_catalog(input)?, &key)?,
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        "expected manifest, repository, revision, group, default, or catalog",
                    ));
                }
            }

            parse_comma(input)?;
        }

        Ok(Self {
            manifest: required(manifest, input, "manifest")?,
            repository,
            revision,
            group: required(group, input, "group")?,
            default: required(default, input, "default")?,
            catalog: required(catalog, input, "catalog")?,
        })
    }
}

fn parse_page_catalog(input: ParseStream<'_>) -> Result<PageCatalogInvocation> {
    let descriptor = input.parse()?;
    let content;
    braced!(content in input);

    let mut path = None;
    let mut component = None;
    let mut custom = None;
    while !content.is_empty() {
        let key: Ident = content.parse()?;
        content.parse::<Token![:]>()?;
        match key.to_string().as_str() {
            "path" => set_once(&mut path, content.parse()?, &key)?,
            "component" => set_once(&mut component, content.parse()?, &key)?,
            "custom" => {
                let entries;
                bracketed!(entries in content);
                let values = entries
                    .parse_terminated(Expr::parse, Token![,])?
                    .into_iter()
                    .collect();
                set_once(&mut custom, values, &key)?;
            }
            _ => {
                return Err(syn::Error::new(
                    key.span(),
                    "expected path, component, or custom",
                ));
            }
        }
        parse_comma(&content)?;
    }

    Ok(PageCatalogInvocation {
        descriptor,
        path: required(path, &content, "path")?,
        component: required(component, &content, "component")?,
        custom: custom.unwrap_or_default(),
    })
}

fn parse_group(input: ParseStream<'_>) -> Result<GroupInvocation> {
    let ty = input.parse()?;
    let content;
    braced!(content in input);

    let mut mappings = Vec::new();
    while !content.is_empty() {
        let id = content.parse()?;
        content.parse::<Token![=>]>()?;
        mappings.push(GroupMapping {
            id,
            value: content.parse()?,
        });
        parse_comma(&content)?;
    }

    Ok(GroupInvocation { ty, mappings })
}

fn parse_examples(input: ParseStream<'_>) -> Result<Vec<ExampleInvocation>> {
    let mut examples = Vec::new();

    while !input.is_empty() {
        let module: Ident = input.parse()?;
        if module.to_string().starts_with("r#") {
            return Err(syn::Error::new(
                module.span(),
                "raw identifiers are not supported for Example modules",
            ));
        }
        let content;
        braced!(content in input);

        let mut slug = None;
        let mut title = None;
        let mut description = None;
        while !content.is_empty() {
            let key: Ident = content.parse()?;
            content.parse::<Token![:]>()?;
            match key.to_string().as_str() {
                "slug" => set_once(&mut slug, content.parse()?, &key)?,
                "title" => set_once(&mut title, content.parse()?, &key)?,
                "description" => set_once(&mut description, content.parse()?, &key)?,
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        "expected slug, title, or description",
                    ));
                }
            }
            parse_comma(&content)?;
        }

        examples.push(ExampleInvocation {
            slug: slug.unwrap_or_else(|| LitStr::new(&module.to_string(), module.span())),
            module,
            title,
            description: required(description, &content, "description")?,
        });
        parse_comma(input)?;
    }

    Ok(examples)
}

fn parse_comma(input: ParseStream<'_>) -> Result<()> {
    if input.peek(Token![,]) {
        input.parse::<Token![,]>()?;
    } else if !input.is_empty() {
        return Err(input.error("expected `,`"));
    }
    Ok(())
}

fn set_once<T>(slot: &mut Option<T>, value: T, key: &Ident) -> Result<()> {
    if slot.replace(value).is_some() {
        return Err(syn::Error::new(
            key.span(),
            format!("`{key}` may only be declared once"),
        ));
    }
    Ok(())
}

fn required<T>(value: Option<T>, input: ParseStream<'_>, name: &str) -> Result<T> {
    value.ok_or_else(|| input.error(format!("missing required `{name}` field")))
}

pub(crate) fn valid_group_id(value: &str) -> bool {
    !value.is_empty()
        && value.split('-').all(|part| {
            !part.is_empty()
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        })
}

/// Converts a Rust Example module identifier to its generated section name.
#[must_use]
pub fn example_section_name(module: &str) -> String {
    let upper_camel = module
        .split('_')
        .filter(|part| !part.is_empty())
        .map(|part| {
            let mut characters = part.chars();
            characters
                .next()
                .into_iter()
                .flat_map(char::to_uppercase)
                .chain(characters)
                .collect::<String>()
        })
        .collect::<String>();

    format!("{upper_camel}ExampleSection")
}

/// Converts a Rust identifier to the default human-readable Example title.
#[must_use]
pub fn sentence_case(identifier: &str) -> String {
    let words = identifier.replace('_', " ");
    let mut characters = words.chars();

    match characters.next() {
        Some(first) => first.to_uppercase().chain(characters).collect(),
        None => words,
    }
}
