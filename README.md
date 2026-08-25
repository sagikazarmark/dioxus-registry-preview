# dioxus-registry-preview

**Convention-driven tooling for Dioxus Component Registry Preview applications.**

## Features

- **Validated authoring model.** Discover Component manifests, source, READMEs, Examples, and site catalogs without compiling Dioxus.
- **Composable generated pieces.** Generate metadata, Example sections, README adapters, Component pages, and a complete ordered catalog.
- **Isolated default chrome.** Mount a complete routed Preview whose namespaced styles do not reset or restyle Consumer content.
- **Versioned browser protocol.** Drive generic Playwright coverage from stable page, Example, catalog, and theme markers.

**Building a Registry site?** Start with the [step-by-step Registry author
guide](docs/getting-started.md), from one Component through validation and
browser tests.

The repository contains three Rust packages:

- `dioxus-registry-preview-core`: Dioxus-free discovery, validation, Markdown rendering, and catalog models.
- `dioxus-registry-preview`: the public facade, macros, validation exports, and isolated default chrome.
- `dioxus-registry-preview-macros`: the facade's procedural-macro implementation.

It also ships a generic Playwright helper package and a fixture Registry that exercises the complete external-consumer boundary.

## Quick Start

Add the facade to the Preview-only part of a Registry:

```toml
[dependencies]
dioxus-registry-preview = "0.3.0"

dioxus = { version = "0.7.0", default-features = false, features = ["launch", "lib", "web"] }
```

Set the Registry package's Cargo `repository`, invoke `component!` beside each Component's excluded documentation source, and invoke `component_pages!` at the site's catalog. The catalog macro emits `APP_CATALOG` with its authoritative group-ID mapping, Registry facts, and installation source; once the Consumer's rendering kind implements `chrome::AppPage`, that generated catalog is a complete default site:

```rust
use dioxus::prelude::*;

mod pages;

fn main() {
    dioxus::launch(|| rsx! {
        dioxus_registry_preview::chrome::App {
            catalog: pages::APP_CATALOG,
        }
    });
}
```

`App` supplies dynamic routing over catalog paths, a generated installation page at `/installation`, a default indexed home at `/`, document titles, a not-found page, branding, navigation, the default light/dark switcher, and all DOM protocol writers. The installation page derives its worked Component command, `--all` command, `Dioxus.toml` configuration, and ordered install names from Registry facts. Authored `/installation` or `/` descriptors replace the corresponding generated page. The [consumer contract](docs/adr/0001-versioned-consumer-contract.md) defines generated sections, stable group IDs, invocation-relative discovery, catalog descriptors, and the DOM protocol.

The customization ladder is:

1. Use `chrome::App` for the complete default site.
2. Keep a Consumer shell and compose `Styles`, `PageCatalogManifest`, `ThemeSwitcherManifest` or `ThemeSwitcher`, and `Sidebar`.
3. Replace all presentation while retaining the generated pieces and catalog contract.

Generic validation is available through the same facade for ordinary Rust tests:

```rust
use dioxus_registry_preview::validation::{MarkdownOptions, load_site_from_catalog};

let validation = load_site_from_catalog(
    "src/preview/pages.rs".as_ref(),
    &MarkdownOptions::default(),
);
assert!(validation.diagnostics.is_empty());
```

The facade's default site, Example, README, source, navigation, and shell chrome uses small namespaced stylesheets scoped to `data-registry-preview-chrome`. It requires no Tailwind installation and can be replaced through the existing adapter paths or by mounting a Consumer-owned shell.

The Playwright helpers are available as an npm package:

```json
{
  "devDependencies": {
    "@sagikazarmark/dioxus-registry-preview-playwright": "0.3.0"
  }
}
```

See [compatibility](docs/compatibility.md), [browser helpers](docs/browser-helpers.md), and [default chrome](docs/default-chrome.md).

## Development

```shell
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
```

The browser acceptance test additionally requires the Dioxus CLI, `wasm32-unknown-unknown`, npm, and Playwright's Chromium:

```shell
npm ci
npm ci --prefix tests/browser
dx build --package docs-registry-fixture --bin fixture-preview --features web --platform web --release
dx build --package docs-registry-fixture --bin custom-home-preview --features web --platform web --release
npm test --prefix tests/browser
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
