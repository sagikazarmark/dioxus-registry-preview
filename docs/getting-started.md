# Build Registry documentation

This guide starts with one Dioxus Component and ends with a rendered,
browser-tested Registry documentation site. It is for Registry authors. You do
not need to understand the framework's internal architecture or read an ADR to
follow it.

The example uses `src/components/button` as the Component member. The same
conventions apply when a Registry keeps its `components/<name>` directories
somewhere else: member paths are relative to the root Registry manifest, while
macro paths are relative to the Rust source file containing the invocation.

## Before you start

Use Rust 1.88 or newer and Dioxus 0.7.x. The Dioxus CLI patch should match the
resolved Dioxus dependency patch; this guide currently resolves 0.7.10. Install
that CLI and the web target:

```shell
cargo install dioxus-cli --version 0.7.10
rustup target add wasm32-unknown-unknown
```

The completed layout is:

```text
.
|-- Cargo.toml
|-- component.json
|-- src
|   |-- lib.rs
|   |-- components
|   |   |-- mod.rs
|   |   `-- button
|   |       |-- README.md
|   |       |-- component.json
|   |       |-- component.rs
|   |       |-- mod.rs
|   |       `-- docs
|   |           |-- mod.rs
|   |           `-- examples
|   |               `-- overview.rs
|   `-- preview
|       |-- example.rs
|       |-- main.rs
|       `-- pages.rs
|-- tests
|   |-- registry_preview.rs
|   `-- browser
|       |-- package.json
|       |-- playwright.config.ts
|       `-- registry.spec.ts
|-- README.md
|-- LICENSE-APACHE
`-- LICENSE-MIT
```

## 1. Configure the Registry crate

Start a library package named `acme-registry`. Keep documentation dependencies
behind a Preview-only feature so installing a Component does not pull the site
into a user's application.

```toml
[package]
name = "acme-registry"
version = "0.1.0"
description = "Acme Component Registry"
edition = "2024"
rust-version = "1.88"
license = "MIT OR Apache-2.0"
repository = "https://github.com/acme/acme-registry"
keywords = ["dioxus", "components", "registry"]
categories = ["web-programming"]
include = [
    "/src/lib.rs",
    "/src/components/**/*.rs",
    "!/src/components/**/docs/**/*.rs",
    "/README.md",
    "/LICENSE-APACHE",
    "/LICENSE-MIT",
]

[[bin]]
name = "preview"
path = "src/preview/main.rs"
required-features = ["preview"]

[features]
default = []
preview = [
    "dep:dioxus-registry-preview",
    "dep:web-sys",
    "dioxus/launch",
    "dioxus/web",
]

[dependencies]
dioxus-registry-preview = { version = "0.1.0", optional = true }

dioxus = { version = "0.7.0", default-features = false, features = ["lib"] }
web-sys = { version = "0.3.77", features = ["Location", "Window"], optional = true }
```

Expose the installable namespace from `src/lib.rs`:

```rust
pub mod components;
```

Declare the Component in `src/components/mod.rs`:

```rust
pub mod button;
```

The `include` list packages the installable Rust but omits every
`docs/**/*.rs` file and the Preview binary. Independently, the Component
manifest in the next step keeps the same authoring files out of `dx components
add`. These are two separate packaging boundaries.

## 2. Author one Component

Create `src/components/button/mod.rs` without declaring its `docs` directory:

```rust
mod component;

pub use component::*;
```

Create the installable source at `src/components/button/component.rs`:

```rust
use dioxus::prelude::*;

#[component]
pub fn Button(children: Element) -> Element {
    rsx! { button { class: "button", {children} } }
}
```

Create `src/components/button/component.json`:

```json
{
  "name": "button",
  "description": "An action control for forms and dialogs.",
  "exclude": ["component.json", "README.md", "docs"]
}
```

The `name` is the stable page and installation ID. The three `exclude` entries
are required. They prevent Registry metadata, narrative documentation, and
Preview-only Rust from being installed with the Component.

Make `name` a valid Rust module identifier, such as `button` or `date_picker`.
Keeping it equal to the member directory avoids maintaining separate discovery
and installation names, but it may differ: `component_pages!` discovers the
member by its final directory while generated commands use the manifest name.
`dx components add` creates an installation directory and a `pub mod`
declaration from `name`; values such as `date-picker` pass Registry
documentation validation but produce an invalid Rust module declaration during
installation.

Create `src/components/button/README.md` with an H1 and an introductory
paragraph before any body sections:

```markdown
# Button

Buttons let a user trigger an action.

## When to use it

Use a button for an immediate action, not for navigation.
```

The H1 becomes the page title. The Component manifest's `description` becomes
the catalog description. When the README is rendered, its H1 and introductory
paragraph are omitted because the site already has that metadata; the
remaining Markdown is rendered as the README section.

## 3. Register the Registry

The root `component.json` is the only Component membership list. Its order is
also the generated Component page order:

```json
{
  "name": "acme-registry",
  "description": "Acme Dioxus Components.",
  "members": [
    "src/components/button"
  ]
}
```

Each member must be a normalized relative path. Its final directory name must
be a Rust module identifier because the catalog macro generates that module.
It may differ from the Component manifest's `name`; in that case the former is
the Preview module while the latter is the page and installation ID.

## 4. Write a compiling Example

Create `src/components/button/docs/examples/overview.rs`:

```rust
use crate::components::button::Button;
use dioxus::prelude::*;

#[component]
pub fn Example() -> Element {
    rsx! { Button { "Save changes" } }
}
```

Every declared Example is a Rust module under `docs/examples`. It must expose a
public, zero-prop Dioxus component named `Example`. The macro compiles that
component and stores the exact same file as displayed source, so rendered and
printed Examples cannot drift apart. The default chrome shows each Example
behind a Preview / Code tab pair; the Code tab is that file.

To syntax-highlight the Code tab, enable the facade's `syntax-highlighting`
feature:

```toml
dioxus-registry-preview = { version = "0.1.0", optional = true, features = ["syntax-highlighting"] }
```

The macro then highlights every Example at compile time and the chrome renders
it through `dioxus-code`. The feature adds tree-sitter's C runtime to the
Preview build, so the machine building it needs a C compiler that targets
`wasm32-unknown-unknown`; on macOS that is LLVM `clang` from Homebrew or Nix,
selected with `CC_wasm32_unknown_unknown`, because Apple's bundled clang cannot.
Without the feature nothing compiles C and the Code tab shows plain text.

Create `src/components/button/docs/mod.rs`:

```rust
dioxus_registry_preview::component! {
    group: "actions",
    render_readme: true,
    examples: {
        overview {
            description: "A Button that submits the user's changes.",
        },
    },
}
```

The invocation belongs as a top-level item in `docs/mod.rs`. By default it
loads these invocation-relative paths:

| Input | Path from `docs/mod.rs` |
| --- | --- |
| Component manifest | `../component.json` |
| Component source | `../component.rs` |
| README | `../README.md` |
| Example module | `examples/<module>.rs` |

Use `component_root: "../somewhere"` only when the manifest, source, and README
have an unusual location. Examples always remain under the invoking module's
`examples` directory.

`group` and a nonempty `examples` block are required. Each Example requires a
`description`. Its slug defaults to the module name and its title defaults to
that name in sentence case. Override them when display metadata should differ:

```rust
examples: {
    destructive_action {
        slug: "delete",
        title: "Delete an item",
        description: "A destructive Button with a confirmation step.",
    },
},
```

The macro generates ordinary, composable Rust items in this module:

| Generated item | Purpose |
| --- | --- |
| `DOCUMENTATION` | Component, README, and ordered Example metadata |
| `GROUP_ID` and `LISTED` | Stable catalog policy |
| `ReadmeSection` | README adapter call when `render_readme: true` |
| `OverviewExampleSection` | Literal `overview::Example {}` plus its adapter |
| `DocumentationPage` | README followed by all Example sections by default |

A multiword Example such as `destructive_action` generates
`DestructiveActionExampleSection`. Display slug or title changes never rename
the Rust item.

## 5. Build the page catalog

The macro defaults to adapter functions at `crate::example`. Satisfy that
convention in `src/preview/example.rs` by re-exporting the facade's isolated
default chrome:

```rust
pub(crate) use dioxus_registry_preview::chrome::{ExampleSection, ReadmeSection};
```

Create `src/preview/pages.rs`:

```rust
use dioxus::prelude::*;

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum ComponentGroup {
    Actions,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub(crate) enum PageKind {
    Component(ComponentPage),
}

impl dioxus_registry_preview::chrome::AppPage for PageKind {
    fn kind(self) -> &'static str {
        match self {
            Self::Component(_) => "generated-component",
        }
    }

    fn render(self) -> Element {
        match self {
            Self::Component(page) => page.view(),
        }
    }
}

pub(crate) type PageDescriptor =
    dioxus_registry_preview::PageDescriptor<ComponentGroup, PageKind>;

dioxus_registry_preview::component_pages! {
    manifest: "../../component.json",
    group: ComponentGroup {
        "actions" => ComponentGroup::Actions,
    },
    default: button,
    catalog: PageDescriptor {
        path: "/components",
        component: PageKind::Component,
        custom: [],
    },
}
```

`manifest` is relative to `src/preview/pages.rs`. The group mapping must cover
every `group` in every `component!` invocation. Its string IDs are durable
authoring data; the Rust enum and variants can be reorganized without editing
all Component documentation.

This invocation is also the full-site validation entry point. It is the single
source for the root manifest path, ordered group IDs, default Component, and any
installation-source override. `default` is the final member directory/module
name, not the Component manifest's page ID; the generated installation command
uses that member's Component manifest `name`.

By default, the Registry repository comes from `[package].repository`, as set in
step 1. Add `repository: "https://..."` to `component_pages!` only when the
Preview package's Cargo metadata points somewhere else. Add `revision: "v1.2.3"`
when install commands and `Dioxus.toml` should pin one Git revision. A Registry
in a monorepo subdirectory needs a separate repository whose root contains
`component.json`; `dx components add --git` does not accept a repository
subdirectory.

The catalog macro emits `PAGE_CATALOG`, the lower-level ordered descriptor
slice, and `APP_CATALOG`, which pairs that slice with the authoritative group
IDs and Registry facts needed by the default site. Handwritten descriptors in
`custom` come first, followed by generated Component pages in root-manifest
order. The facade-owned installation and home entries augment only `App`'s
runtime catalog; they do not change `PAGE_CATALOG`.

Generated Component pages are always enabled for browser testing. Setting
`listed: false` in `component!` removes a page from navigation but does not
remove it from browser coverage.

## 6. Launch the default site

Create `src/preview/main.rs`:

```rust
use dioxus::prelude::*;

pub use acme_registry::components;

mod example;
mod pages;

#[allow(unused_imports)]
pub(crate) use pages::example_modules as examples;

fn main() {
    dioxus::launch(Preview);
}

#[component]
fn Preview() -> Element {
    rsx! {
        dioxus_registry_preview::chrome::App {
            catalog: pages::APP_CATALOG,
            title: "Acme Registry",
        }
    }
}
```

The `components` re-export gives Examples the same
`crate::components::<name>` path they have in the Registry package and after
installation. The `example_modules as examples` alias gives one Example a
stable path to another Component's Example when fixtures need to be shared:

```rust
use crate::examples::button::overview::Example as ButtonFixture;
```

`chrome::App` supplies exact catalog routing, document titles, a not-found
page, branding, listed-page navigation, a light/dark switcher, isolated styles,
and the complete browser-helper protocol. It generates `/installation` from the
Registry facts, with a worked Button command, `--all`, persistent
`Dioxus.toml` configuration, and the ordered Component names. When the catalog
does not claim `/`, `App` renders a default home there with the Registry name,
root manifest description, and an index of every listed page. The home action
links to installation. These are rendered pages, not redirects; generated
Button documentation remains at `/components/button`.

Compile the Preview, then run it:

```shell
cargo check --features preview --bin preview
dx serve --package acme-registry --bin preview --features preview --platform web
```

Open `http://127.0.0.1:8080/installation`. If `dx serve` reports another
address, use that address instead. The worked command is:

```shell
dx components add button --git 'https://github.com/acme/acme-registry'
```

## 7. Validate every authoring convention

Macro expansion reports the first error needed to compile the current page or
catalog. Full-site validation reports independent errors together, including
packaging and layout policy that macro expansion does not enforce.

Create `tests/registry_preview.rs`:

```rust
use std::path::Path;

use dioxus_registry_preview::validation::{MarkdownOptions, load_site_from_catalog};

#[test]
fn registry_documentation_is_valid() {
    let catalog = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/preview/pages.rs");
    let validation = load_site_from_catalog(&catalog, &MarkdownOptions::default());
    let diagnostics = validation
        .diagnostics
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("\n");

    assert!(validation.diagnostics.is_empty(), "{diagnostics}");
}
```

Run it whenever a Component's files or membership change:

```shell
cargo test --features preview --test registry_preview --locked
```

Diagnostics use stable `RDOC###` codes. The [convention and diagnostic
reference](#convention-and-diagnostic-reference) below explains what each
author-facing failure means.

## 8. Add generic browser coverage

The Playwright helper and Rust facade consume the same versioned DOM protocol.
Pin both to the same release. The Cargo manifest in step 1 uses
`dioxus-registry-preview = { version = "0.1.0", optional = true }`, and
`tests/browser/package.json` uses the matching helper release:

```json
{
  "name": "acme-registry-browser-tests",
  "private": true,
  "type": "module",
  "scripts": {
    "test": "playwright test"
  },
  "devDependencies": {
    "@playwright/test": "1.62.1",
    "@sagikazarmark/dioxus-registry-preview-playwright": "0.1.0"
  }
}
```

Do not independently float one side of the pair. To track unreleased changes,
pin both dependencies to the same repository revision instead:

```toml
dioxus-registry-preview = { git = "https://github.com/sagikazarmark/dioxus-registry-docs", rev = "<tooling-revision>", optional = true }
```

```json
"@sagikazarmark/dioxus-registry-preview-playwright": "github:sagikazarmark/dioxus-registry-docs#<tooling-revision>"
```

Create `tests/browser/playwright.config.ts`:

```typescript
import { defineConfig, devices } from "@playwright/test";

export default defineConfig({
  testDir: ".",
  use: {
    baseURL: "http://127.0.0.1:8080",
    ...devices["Desktop Chrome"],
  },
});
```

Create `tests/browser/registry.spec.ts`:

```typescript
import { expect, test } from "@playwright/test";

import {
  addresses,
  example,
  openPreview,
} from "@sagikazarmark/dioxus-registry-preview-playwright";

test("every browser-enabled page renders under every baseline theme", async ({ page }) => {
  const discovered = await addresses(page);

  expect(discovered).toEqual([
    { page: { id: "registry-home", path: "/" }, theme: "light" },
    { page: { id: "registry-home", path: "/" }, theme: "dark" },
    { page: { id: "registry-installation", path: "/installation" }, theme: "light" },
    { page: { id: "registry-installation", path: "/installation" }, theme: "dark" },
    { page: { id: "button", path: "/components/button" }, theme: "light" },
    { page: { id: "button", path: "/components/button" }, theme: "dark" },
  ]);

  for (const address of discovered) {
    await openPreview(page, address);
  }

  const button = discovered.find(({ page }) => page.id === "button");
  expect(button).toBeDefined();
  await openPreview(page, button!);
  await expect(example(page, "overview")).toBeVisible();
});
```

Install and run the tests while `dx serve` is running:

```shell
npm install --prefix tests/browser
npm --prefix tests/browser exec playwright install -- chromium
npm test --prefix tests/browser
```

`addresses` reads every catalog page with browser testing enabled and combines
it with every baseline theme. This includes the generated installation and
default home pages, plus Component pages omitted from navigation. `openPreview` opens one address and waits for
its protocol markers; `example` scopes assertions to the rendered content
rather than its source code.

For CI, build first with the exact target selection:

```shell
dx build --package acme-registry --bin preview --features preview --platform web --release
```

Serve `target/dx/preview/release/web/public` with fallback to `index.html` for
unknown file paths before starting Playwright. The SPA fallback is required
because browser tests navigate directly to catalog paths.

## Choose your customization level

The generated pieces and catalog are stable seams. Choose the first rung that
meets the site's needs instead of replacing presentation preemptively.

### 1. Default site

Use `chrome::App` as shown above.

| Responsibility | Result |
| --- | --- |
| You write | Component files, `component!`, catalog types, `AppPage`, `component_pages!`, adapter re-exports, and the launch function |
| You inherit | Generated installation and home pages, routing, titles, not-found handling, branding, `Sidebar`, default light/dark themes, protocol manifests, default adapters, and isolated styles |
| You give up | A public static route enum, custom shell layout, custom header/sidebar structure, and themes beyond the default light/dark pair |

`App` accepts only `APP_CATALOG` and an optional title. Paths are matched
exactly. The brand links to `/`. If no authored descriptors claim them, App
renders its listed, browser-tested `registry-installation` page at
`/installation` and its unlisted, browser-tested `registry-home` page at `/`.
The home action links to installation. Supplying an authored descriptor at
either path replaces that generated page without another option.

### 2. Your shell, our parts

Keep your router and layout, but compose facade-owned protocol and chrome
parts: `Styles`, `PageCatalogManifest`, `ThemeSwitcherManifest`,
`ThemeSwitcher`, `Sidebar`, or the smaller `Shell` composition.

| Responsibility | Result |
| --- | --- |
| You write | Static routes, current-page selection, titles, not-found behavior, header/layout, catalog-to-`CatalogEntry` conversion, links, and any custom theme control |
| You inherit | Protocol writers, optional default content styles, optional default adapters, optional `Sidebar`, and optional default light/dark switching |
| You give up | `App` routing, automatic titles, automatic query preservation, and automatic catalog conversion |

Mount protocol components rather than duplicating their hidden markup. A
custom N-theme switcher mounts `ThemeSwitcherManifest` from the same theme
model as its visible controls. Keep all choices attached to the DOM, mark a
nonempty baseline subset, keep the current choice checked, and synchronize the
`theme` query parameter. Use `APP_CATALOG.pages()` and
`APP_CATALOG.group_id(page)` when converting descriptors. Installation facts
remain available through `APP_CATALOG.registry()` for rendering the facade's
`InstallationPage` or a Consumer replacement.

`Styles` only installs facade CSS. Use `Shell`, or retain both
`data-registry-preview-chrome="true"` and `class="rpv-shell"` on your own chrome
root, if you want those isolated styles to apply. `Sidebar` needs the current
page ID, `CatalogEntry` values, a link builder, and optionally a client-side
navigation callback.

### 3. Your everything

Retain the catalog slice and generated pieces while replacing all presentation,
as a fully customized Registry such as the daisyUI Registry does.

| Responsibility | Result |
| --- | --- |
| You write | Static routes, page shell, navigation, themes, CSS, adapters, custom page assembly, titles, indexes, and not-found behavior |
| You inherit | `PAGE_CATALOG`, `ComponentPage::view`, `DOCUMENTATION`, generated README/Example sections, literal Example calls, stable group IDs, and validation |
| You give up | Every facade-owned presentation and behavior guarantee you replace |

Use `page: CustomPage` in `component!` to reorder, omit, or interleave generated
sections while keeping their metadata and literal Example calls:

```rust
use dioxus::prelude::*;

#[component]
fn CustomPage() -> Element {
    rsx! {
        OverviewExampleSection {}
        p { "Registry-specific guidance between generated sections." }
        ReadmeSection {}
    }
}

dioxus_registry_preview::component! {
    group: "actions",
    render_readme: true,
    page: CustomPage,
    examples: {
        overview { description: "The primary Button style." },
    },
}
```

If generic browser helpers remain part of the site, continue mounting the
facade's protocol writers and preserve the semantic promises of `data-page`,
catalog, Example, and theme markers. Classes, labels, and visual structure are
not part of that protocol.

### Code without a preview

`chrome::CodeBlock` shows code that has no rendered Example, such as a manifest
or a configuration snippet. Plain text works in every build:

```rust
use dioxus_registry_preview::chrome::CodeBlock;

rsx! {
    CodeBlock { content: include_str!("../component.json"), label: "component.json" }
}
```

With `syntax-highlighting` enabled, `CodeBlock` also accepts a highlighted
source. Add `dioxus-code` to the Preview's own dependencies and produce one at
compile time; the macro looks the crate up in your `Cargo.toml`, so the facade's
re-export is not enough:

```rust
use dioxus_code::{CodeOptions, Language, code_str};
use dioxus_registry_preview::chrome::CodeBlock;

rsx! {
    CodeBlock {
        content: code_str!(
            "use crate::examples::button::overview::Example;",
            CodeOptions::builder().with_language(Language::Rust)
        ),
        label: "Importing an Example",
    }
}
```

`dioxus-code` gates every non-Rust language behind a `lang-*` feature that also
enables its runtime parser, so highlighted JSON or TOML costs more than plain
text. Keep manifests plain unless the highlighting is worth that weight.

## Convention and diagnostic reference

Run full-site validation first when setup fails: it aggregates independent
problems. Macro expansion reports only the first relevant diagnostic, and Rust
then reports type or module errors for conventions that have no `RDOC` code.

### Catalog and discovered inputs

| Convention | Failure you will see |
| --- | --- |
| The catalog source is readable, valid Rust with a `component_pages!` invocation | `RDOC023` for read failure, `RDOC024` for source or invocation parse failure, `RDOC025` when the invocation is absent |
| `manifest` identifies a readable, valid root manifest relative to the catalog source | `RDOC004` for read failure, `RDOC005` for parse failure |
| The root manifest has a nonempty Registry name | `RDOC040: the Registry manifest name must be nonempty` |
| The root manifest has at least one member | `RDOC006: the Registry manifest lists no members` |
| Member paths use `/`, are normalized and relative, and end in non-raw Rust module names | `RDOC007: Registry member ... must be a normalized relative path` or `does not end in a Rust module name` |
| Final member directory/module names are unique | `RDOC008: duplicate Registry member module ...` |
| Every Component manifest is readable JSON with `name` and `description` strings | `RDOC009` for read failure, `RDOC010` for parse failure |
| Component `name` is a nonempty lowercase page/install slug | `RDOC011: Component name ... must be a nonempty lowercase slug` |
| Component source is readable, valid Rust | `RDOC012` for read failure, `RDOC013` for parse failure |
| Component source has a top-level public function with unqualified `#[component]` | `RDOC014: ... declares no public #[component] function` |
| README is readable and starts with an H1 plus introductory paragraph | `RDOC015` for read failure, `RDOC016: ... must start with an H1 and an introductory paragraph` |
| Every declared Example file exists | `RDOC017: cannot read .../docs/examples/<module>.rs` |

### Documentation module and macro input

| Convention | Failure you will see |
| --- | --- |
| Group IDs are nonempty lowercase kebab-case | `RDOC018: group ID ... must be nonempty lowercase kebab-case` |
| Group IDs are unique in the catalog mapping | `RDOC019: duplicate group ID` |
| Every `component!` declares at least one Example | `RDOC020: a component page must declare at least one example` |
| Explicit Example slugs are nonempty and unique within the page | `RDOC021: example slug cannot be empty` or `RDOC022: duplicate example slug` |
| `docs/mod.rs` is readable and valid Rust | `RDOC023` for read failure, `RDOC024` for module or invocation parse failure |
| `component!` is a top-level item in `docs/mod.rs` | `RDOC025: ... contains no component! invocation` when full-site validation cannot discover it |
| Every Component group appears in the site/catalog mapping | `RDOC026: unknown group ID ...; valid group IDs: ...` |
| Component manifest excludes `component.json`, `README.md`, and `docs` | `RDOC027: Component manifest does not exclude: ...`; only full-site validation enforces this |
| A Component does not retain the old `docs.md` or top-level `examples` layout | `RDOC028: Component still uses the superseded docs.md/examples layout` |
| Every member has `docs/mod.rs` | `RDOC029: Component docs has no mod.rs` |
| Every member has a `docs/examples` directory | `RDOC030: Component docs has no examples directory` |
| `page:` names a same-module, zero-prop `#[component]` function | `RDOC031: page ... must name a zero-prop #[component] function in this module`; full validation catches this before ordinary generated-call errors |
| `default` names a member directory/module | `RDOC032: default Component ... is not a Registry member` |
| Component manifest page IDs are unique across members | `RDOC033: duplicate Component page ID ...` |
| `RDOC034` and `RDOC035` remain reserved after removal of section-pattern validation | These identifiers are not emitted or available for reuse |
| The catalog group mapping is nonempty | `RDOC036: group mapping must declare at least one ID` |
| A Registry repository is configured by `repository:` or the invoking package's Cargo metadata | `RDOC037: no Registry repository is configured ...` during macro expansion |
| The Registry repository is an absolute `git`, `http`, `https`, or `ssh` URL | `RDOC038: Registry repository must be an absolute ... URL` |
| An optional Registry revision is nonempty, has no whitespace, and does not start with `-` | `RDOC039: Registry revision must be a nonempty Git revision ...` |
| `component_root` and `component_pages!` manifest paths are relative to their invocation files | `RDOC007: component_root must be relative to docs/mod.rs` or `manifest must be relative to the macro invocation` |

Unknown, duplicate, or missing macro fields are syntax errors such as `missing
required 'examples' field` rather than stable `RDOC` diagnostics.

### Rust, Dioxus, and packaging conventions

| Convention | Failure you will see |
| --- | --- |
| Consumers depend on `dioxus-registry-preview`, not its macros package | Macro imports or generated facade paths fail; the macros package is an implementation detail |
| Macro invocations stay in Consumer-owned physical source files | `cannot locate the macro invocation`, or discovery resolves relative to a forwarding macro instead of your files |
| `crate::example::ExampleSection` accepts `documentation: ExampleDocumentation` and `children: Element` | Ordinary unresolved-path, missing-prop, or type errors from generated RSX |
| `crate::example::ReadmeSection` accepts `html: &'static str` | Ordinary unresolved-path, missing-prop, or type errors from generated RSX |
| A declared Example exports a public zero-prop `Example` component | Ordinary parse, visibility, missing-item, or required-prop compiler errors at the generated literal `module::Example {}` call |
| Generated item names do not collide with handwritten names or each other | Ordinary duplicate-definition errors for `DocumentationPage`, `ReadmeSection`, constants, modules, or generated Example section names |
| Example modules use ordinary, non-raw identifiers | Macro parsing rejects forms such as `r#type`; generated section names and source paths use ordinary identifier spelling |
| `component_pages!` uses a `PageDescriptor<Group, Kind>` alias and a matching Component constructor | Ordinary field or type mismatch errors in generated catalog code |
| Group and page-kind values satisfy `Clone + Copy + PartialEq + 'static` for `chrome::App` | Trait-bound errors at generated catalog or `chrome::App` use |
| The page kind implements `chrome::AppPage` and generated variants call `ComponentPage::view()` | A missing `AppPage` trait-bound error, or the Consumer's own non-rendering behavior |
| `pages::example_modules` is aliased as `crate::examples` before cross-Component Example imports use it | Ordinary unresolved `crate::examples` import errors |
| `src/components/mod.rs` publicly declares each installable member module | Ordinary unresolved `crate::components::<name>` imports in the Registry and Preview |
| Each installable Component `mod.rs` declares `mod component` and publicly re-exports its items | Ordinary missing-module, private-item, or unresolved Component imports |
| Installable `mod.rs` never declares `docs` | Normal builds or installed Components try to compile Preview-only modules and fail on missing dependencies/files |
| Cargo packaging omits Preview/docs Rust and each Component manifest has all three install exclusions | `cargo package`, Registry validation (`RDOC027`), or installed-Component compilation exposes the packaging mismatch |
| Preview re-exports the Registry's installed namespace at the binary crate root | Example imports such as `crate::components::button` fail only in Preview or only after installation |
| Component manifest `name` and the final member directory are valid Rust module identifiers | Documentation validation accepts some lowercase slugs that make `dx components add` emit an invalid `pub mod <name>;` declaration; the installed crate then fails to parse. The two identifiers may differ: the directory is the Preview module and `name` is the page/install ID |

Rust 1.88 is required because invocation-relative discovery uses
`proc_macro::Span::local_file()`. All discovered files are also emitted through
`include_str!` or generated module paths, so changing a manifest, README,
Component source, or Example causes Cargo to rebuild the Preview.

## Pinning and compatibility

These versions are one consumer contract, not independent choices:

| Surface | Current supported line | Pinning rule |
| --- | --- | --- |
| Rust facade, core, and macros packages | Unreleased `0.3.x`, released together | Pin one repository revision until publication, then use an exact facade version |
| Dioxus and Dioxus CLI | `0.7.x`, with matching patches | A supported Dioxus minor change requires a tooling release |
| Rust | 1.88 or newer | Keep the declared package MSRV at least 1.88 |
| Browser DOM markers | protocol version 1 | Pin the Playwright helper to the same repository release/revision as the Rust facade |
| Playwright helper peer | `@playwright/test >=1.50.0` | Pin a concrete Playwright version in the Registry's browser-test package |

There is no runtime negotiation between a site and browser helpers. A protocol
major changes when a generic marker, value, semantic element, or ordering
promise changes. During the `0.x` series, such an incompatible change requires
a new tooling minor line. See the concise [compatibility policy](compatibility.md)
for release semantics and the [browser helper reference](browser-helpers.md)
for helper APIs.
