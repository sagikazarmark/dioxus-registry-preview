# dioxus-registry-preview

Tooling for Dioxus component Registry Preview applications. It discovers consumer-owned authoring files, validates them into an ordinary model, generates composable Dioxus sections and page catalogs, and exposes a versioned DOM protocol to browser helpers.

## Language

**Consumer**: A component Registry and Preview application using these packages. The Consumer owns documentation source, routing, presentation, themes, and policy extensions.

**Component documentation**: One validated Component manifest, source module, README, macro invocation, and ordered set of Examples.

**Generated piece**: A public constant or zero-prop Dioxus component emitted by `component!`. Generated pieces are the interface; `DocumentationPage` is only their default assembly.

**Page catalog**: The single ordered authored descriptor slice from which a Consumer derives navigation, titles, indexes, and browser-test discovery. The default `App` may augment its runtime catalog with the documented Registry home and installation pages without changing that slice.

**DOM protocol**: The versioned `data-*` vocabulary written by a Consumer site and read by generic Playwright helpers.

**Documentation chrome**: Facade-owned default presentation for installation instructions, Examples, READMEs, source, and the site shell. Its namespaced stylesheet is scoped to the chrome root. Consumers may replace the generated page, Example and README adapters, or the shell without copying the defaults.

**Fixture Registry**: The external-consumer acceptance test under `fixtures/registry`. It uses a renamed facade dependency and the facade-owned default chrome without a Consumer stylesheet.

**Isolated chrome**: Default presentation uses only `rpv-*` classes backed by facade-owned CSS scoped below `data-registry-preview-chrome`. It has no reset, root selector, Tailwind, or component-library dependency. Bare element selectors are permitted only below `.rpv-readme`, which contains facade-rendered Markdown. The zero-configuration `App` additionally mounts a document-level stylesheet for the `html`/`body` surface so themes cover the full page; `Shell` and `Styles` never do.
