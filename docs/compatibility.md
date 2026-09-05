# Compatibility

## Dioxus

The `0.3.x` packages support Dioxus `0.7.x`. `dioxus-registry-preview-core` does not depend on Dioxus; the facade's generated API, default chrome, and fixture define the supported Dioxus line.

Before 1.0, changing the supported Dioxus minor line requires a minor release of these packages. After 1.0, dropping a supported Dioxus line requires a major release.

## Rust

The minimum supported Rust version is 1.88. The procedural macro uses `proc_macro::Span::local_file()` for invocation-relative discovery, which establishes that floor. CI runs the Rust suite on 1.88 and current stable.

## Syntax Highlighting

The facade's `syntax-highlighting` feature is off by default. Enabling it adds `dioxus-code` to the Preview and tree-sitter's C runtime to every build target, so a Preview that enables it needs a C compiler able to target `wasm32-unknown-unknown` (for example LLVM `clang` selected through `CC_wasm32_unknown_unknown`; Apple's bundled clang cannot). Example highlighting itself runs at compile time inside `component!`. The `0.x` facade tracks `dioxus-code` `0.1`; the highlighted-source types in the `code` module are re-exports, so a `dioxus-code` breaking release requires a facade minor release before 1.0. Turning the feature on by default would also be a minor release, because it introduces a new build requirement.

## Package Versions

The three Rust packages are released together at the same version. Consumers depend only on `dioxus-registry-preview`; its macros package is an implementation dependency, while core discovery and validation are re-exported through the facade.

## Catalog Syntax

The Consumer's `component_pages!` invocation is the single source for catalog generation and full-site validation. Its manifest path, ordered group-ID mapping, default Component, and optional Registry repository/revision are interpreted by the same facade release; there is no separately versioned documentation configuration schema. When no repository override is present, macro expansion uses the invoking package's `CARGO_PKG_REPOSITORY`.

Adding optional invocation syntax is compatible. Removing or incompatibly changing accepted syntax requires a package-major release. Before 1.0, that means the next minor release.

Public model structs and policy enums are non-exhaustive. Consumers construct authored descriptors
and protocol entries through their `new` constructors and retain wildcard arms when matching policy
enums. This allows a compatible release to add metadata without invalidating downstream struct
literals or exhaustive matches. Example module names use ordinary identifiers; raw identifiers are
rejected before generated names or source paths are derived.

## DOM Protocol

The generic marker vocabulary is protocol version 1. Removing a marker, changing its value or ordering promise, changing theme-address behavior, or moving a marker to a different semantic element creates a new protocol major. Adding an optional marker is compatible.

The facade validates page-catalog and theme-manifest invariants before writing protocol markers.
Empty or duplicate IDs, relative or duplicate paths, unsupported protocol values, a missing theme
baseline, and an ambiguous initial theme are programmer errors rather than malformed DOM output.

During the `0.x` package series, a protocol-major change requires a new minor helper/tooling release. At 1.0 and later, protocol majors and package majors advance together. Consumers should pin the Rust tooling and Playwright helper to one repository release or revision.
