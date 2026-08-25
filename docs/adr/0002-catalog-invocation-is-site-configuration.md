# The catalog invocation is the site configuration

**Status:** accepted; supersedes the configuration decision in ADR 0001.

The Consumer's `component_pages!` invocation is the single source for the root Registry manifest, ordered stable group IDs, default Component, and optional Registry source override and revision. When `repository:` is omitted, macro expansion derives it from the invoking package's `CARGO_PKG_REPOSITORY`; `revision:` is never inferred. Full-site validation parses that Consumer-owned Rust source without compiling Dioxus code, eliminating `registry-docs.json` and the drift possible when validation and generation used separate values. Invocation syntax now follows package compatibility instead of carrying an independent schema version; the retired `RDOC001` through `RDOC003` identifiers remain reserved.
