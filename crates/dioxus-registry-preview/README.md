# dioxus-registry-preview

[![crates.io](https://img.shields.io/crates/v/dioxus-registry-preview?label=crates.io&style=flat-square)](https://crates.io/crates/dioxus-registry-preview)
[![docs.rs](https://img.shields.io/docsrs/dioxus-registry-preview?label=docs.rs&style=flat-square)](https://docs.rs/dioxus-registry-preview)

**Convention-driven, composable tooling for Dioxus Component Registry Preview applications.**

This facade provides the Component and catalog macros, Dioxus-free validation exports, and isolated
default chrome. Consumers depend only on this package; the core and macros packages are
implementation dependencies.

## Quick Start

Add the facade to the Preview-only part of a Registry:

```toml
[dependencies]
dioxus-registry-preview = "0.3.0"

dioxus = { version = "0.7.0", default-features = false, features = ["launch", "lib", "web"] }
```

Mount the generated catalog with the default chrome:

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

Follow the [Registry author guide](https://github.com/sagikazarmark/dioxus-registry-docs/blob/main/docs/getting-started.md)
to create the companion manifests, documentation modules, Examples, catalog, and browser tests.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
