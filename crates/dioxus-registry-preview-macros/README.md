# dioxus-registry-preview-macros

[![crates.io](https://img.shields.io/crates/v/dioxus-registry-preview-macros?label=crates.io&style=flat-square)](https://crates.io/crates/dioxus-registry-preview-macros)
[![docs.rs](https://img.shields.io/docsrs/dioxus-registry-preview-macros?label=docs.rs&style=flat-square)](https://docs.rs/dioxus-registry-preview-macros)

Implementation detail of [`dioxus-registry-preview`](https://github.com/sagikazarmark/dioxus-registry-docs/tree/main/crates/dioxus-registry-preview). Use the facade crate's `component!` and `component_pages!` re-exports instead of depending on this package directly.

The package is published only so the facade can provide procedural macros. It is versioned and
released in lockstep with the facade and core packages.

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or <https://www.apache.org/licenses/LICENSE-2.0>)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or <https://opensource.org/licenses/MIT>)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
