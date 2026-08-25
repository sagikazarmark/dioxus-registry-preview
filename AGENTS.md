# dioxus-registry-preview

## Domain Docs

Read `CONTEXT.md`, `docs/compatibility.md`, and `docs/adr/` before changing a public contract. The DOM marker vocabulary, generated macro pieces, catalog syntax, facade validation exports, and default chrome are versioned consumer interfaces.

## Test Seams

- Core behavior is tested through `dioxus-registry-preview-core` integration tests.
- Macro diagnostics, validation exports, and generated APIs are tested through the facade's tests.
- The renamed-dependency fixture and generic helper package are tested in `tests/browser`.

The macros package is an implementation detail. Consumers and fixtures must enter through the `dioxus-registry-preview` facade.
