# Documentation tooling has a versioned consumer contract

The documentation tooling lives outside each Consumer Registry, while documentation source, the
site, and Registry-specific acceptance tests stay with the Consumer. The boundary is therefore the
generated pieces, stable group IDs, a versioned DOM marker vocabulary, and a small invocation-site
convention. This preserves excluded documentation modules and the guarantee that an Example is the
same file rendered and printed.

## Generated pieces are the interface

`component!` generates pieces that a consuming documentation module may compose itself. The
assembled page is only the default:

- `pub const DOCUMENTATION` contains the Component and Example metadata.
- `pub const GROUP_ID: &'static str` contains the stable group ID from the invocation, and
  `pub const LISTED: bool` contains its listing policy.
- Each declared Example generates a public, zero-prop section component. Its name is the Example's
  Rust module identifier converted from `snake_case` to `UpperCamelCase`, followed by
  `ExampleSection`: `floating_label` generates `FloatingLabelExampleSection`. The name follows the
  module, not the overridable slug or title, so changing display metadata does not rename Rust API.
- Each generated Example section passes that Example's `ExampleDocumentation` value to the
  configured example-section adapter and contains a literal `<module>::Example {}` call. It does
  not store a function pointer or share a component scope, preserving the same-file guarantee.
- With `render_readme: true`, the macro also generates a public, zero-prop `ReadmeSection`. It
  passes `DOCUMENTATION.readme_html` to the configured readme-section adapter. Without that option,
  neither the generated section nor the default page renders the README.
- `pub fn DocumentationPage() -> Element` remains the stable page entry point used by the catalog.
  By default it contains `ReadmeSection`, when enabled, followed by every generated Example section
  in declaration order. It contains no section markup of its own.

An optional `page: CustomPage` names a zero-prop Dioxus component in the invoking module. When it is
present, the generated `DocumentationPage` contains a literal `CustomPage {}` call instead of the
default assembly. `DOCUMENTATION`, `GROUP_ID`, `LISTED`, `ReadmeSection` when enabled, and every
named Example section are still generated. `CustomPage` can reorder or omit generated sections and
interleave handwritten RSX. It can also render consumer-chosen Markdown; the catalog treats that as
a custom page rather than as a lesser form of generated page.

This composable assembly supersedes the original requirement that `DocumentationPage` itself call
every Example. It also narrows the consequence that every registered Example is reachable by
`data-example`: the invocation remains the one registration list and every entry gets metadata and
a literal-call section, but a deliberate `page:` override may leave that section out of the rendered
page. The default assembly still renders every registered Example.

The generated constants, section components, and `DocumentationPage` are `pub` within their
documentation module so manual assembly can name them. The catalog may keep that module
`pub(crate)`, as it does today; the raw Example modules and the generated `example_modules` facade
also remain crate-visible implementation details rather than a public API of the consuming app.

## Group IDs belong to the site catalog

`group:` takes a nonempty lowercase kebab-case string such as `"actions"` or `"data-input"`, not a
Rust type path. The ID is durable authoring data. A site catalog owns the mapping from each ID to
its group value, title, and ordering, so renaming or moving the site's Rust group type does not edit
every Component's documentation invocation.

`component!` records the string without inferring a type from it. The `component_pages!` invocation
declares the site's complete ID-to-value mapping beside the group type:

```rust
group: ComponentGroup {
    "actions" => ComponentGroup::Actions,
    "data-input" => ComponentGroup::DataInput,
    "navigation" => ComponentGroup::Navigation,
    "overlays" => ComponentGroup::Overlays,
    "data-display" => ComponentGroup::DataDisplay,
},
```

That mapping is the catalog's group-ID interface; the group type continues to own titles and
ordering. IDs in the mapping are unique. `component_pages!` loads each Component invocation's group
ID while assembling the site and resolves it through this mapping. An unknown ID is a
compile-time macro diagnostic, not a downstream Rust type error or a runtime fallback. The
diagnostic names the unknown string and the Component documentation invocation that supplied it;
it also reports the mapping's valid IDs. A duplicate catalog ID is likewise a macro diagnostic. A
documentation module can therefore be expanded in isolation, but a site cannot compile a catalog
containing an unresolved group.

## One descriptor slice is the page catalog

`PageDescriptor<Group, Kind>` is the static interface between generated Component discovery and a
consuming site. Each descriptor carries a stable ID, absolute path, title, description, navigation
placement, listing policy, browser-test policy, and a consumer-owned rendering kind. Navigation
placement is either a stable Component group value or a standalone heading. Listing and browser
coverage are separate policies: an unlisted page may and normally does remain browser-tested.
The descriptor and its policy enums are non-exhaustive; Consumers construct descriptors through
`PageDescriptor::new` and retain wildcard match arms so additive metadata remains a compatible API
change.

The `catalog:` block of `component_pages!` names the consumer's descriptor type, Component path
prefix, and constructor for its generated-Component rendering kind. It also accepts the site's
handwritten descriptors. The macro emits one `PAGE_CATALOG` slice with handwritten pages followed by
every Component page in root Registry-manifest order. It also emits `APP_CATALOG`, a small wrapper
that pairs that same slice with the invocation's authoritative group-ID mapping and root Registry
description for `chrome::App`; lower-level catalog consumers gain no `App` trait bounds. Component
IDs, titles, descriptions, group placement, and listing policy come from the ordinary core model;
generated Component pages default to browser testing enabled. The invocation's root Registry
manifest remains the only Component membership list.

The rendering kind is deliberately consumer-owned. This Preview uses generated Component and RSX
kinds; a later Markdown kind can be added without changing the descriptor. A customized Consumer
site's `Route` remains a static `Routable` enum. The zero-configuration `chrome::App` tier instead
uses one facade-private catch-all `Routable` route and selects exact paths from the catalog at runtime.
That dynamic route is an implementation detail of the default tier, not a replacement for a
Consumer's public static route model. Literal generated Component dispatch remains in the macro and
is reached through the Consumer's `AppPage` implementation, so the catalog drives presentation and
policy rather than pretending literal Example calls are data-driven.

## DOM marker contract v1

The following markers are documentation-site protocol version 1 between a rendered documentation
site and its browser helpers. This repository publishes and documents the protocol version.
Removing any generic marker, moving it to an element with a different promise, or changing its
value or ordering semantics requires a protocol-major and tooling-major bump. Adding a new optional
marker or switcher kind is additive. Sites and helpers are pinned to the same protocol major; there
is no runtime version negotiation in the DOM.

Markers carry machine-readable identity and test structure only. They are not styling hooks, and a
consumer may change headings, labels, classes, or surrounding markup without changing them.

**`data-page`.** The consuming site's route/catalog host writes this on the root around the page
currently selected by the address. Neither generated `DocumentationPage`, a `page:` override, nor a
handwritten page component writes it. Its value is that page's stable catalog ID and, for a
Component page, the component path segment. The generic `openPreview` helper and page-specific specs
read it to know that the requested route has rendered. Exactly one current page root has the
requested value; its heading text is presentation and is not an ID.

**`data-page-catalog`.** The consuming site writes this on one hidden root containing its complete
page catalog, including unlisted pages. Generic browser helpers wait for this marker to know the
site has booted, then discover coverage from its entries rather than from visible navigation. The
root is rendered from the same runtime catalog that drives routes, titles, navigation, and page
indexes; it is not a generated file or a second membership list. `chrome::App` derives that runtime
catalog from the authored descriptor slice, prepends its documented `registry-installation` entry
when no descriptor claims `/installation`, and prepends `registry-home` when no descriptor claims
`/`. The generated `PAGE_CATALOG` slice remains unchanged.

**Catalog-entry markers.** The catalog writer puts the descriptor's complete public metadata on
every direct entry, in catalog order. `data-catalog-page` is the page's nonempty stable ID and is
unique in the catalog. `data-path` is its absolute route path and is also unique. `data-title` and
`data-description` carry its presentation metadata. `data-kind` identifies its rendering kind;
version 1 defines `"generated-component"` and `"rsx"`. `data-navigation` is `"standalone"` or
`"group"`, and `data-navigation-value` is respectively the standalone heading or stable Component
group ID. `data-listing` is `"listed"` or `"unlisted"`; `data-browser-test` is `"enabled"` or
`"disabled"`. Generic address discovery reads entries with browser testing enabled regardless of
their listing policy, then opens the recorded path and checks the corresponding `data-page` root.

**`data-example`.** The consumer-owned example-section adapter writes this on the root that encloses
one rendered Example and its documentation chrome. Its value is `ExampleDocumentation::slug` and is
unique within a page. Generic Example lookup and component-specific specs read it as the scope of
that Example. The choice of tabs, panels, source-code markup, and classes below that root belongs to
the adapter and is not part of this contract.

**`data-example-content`.** The example-section adapter writes this on the one descendant of a
`data-example` root that contains the rendered Example rather than its source or documentation
chrome. Generic Example lookup returns this element. Its surrounding tabs, visibility mechanism,
classes, and ARIA implementation remain presentation owned by the adapter.

**`data-switcher`.** The consuming site writes this on the root of a set of visible navigation or
theme choices. Its value is a stable switcher kind; version 1 defines `"component"` and `"theme"`.
The Component switcher contains listed Component pages and is not the coverage manifest. The theme
switcher contains every offered theme. Generic browser helpers read switchers even when presentation
hides them. There is at most one switcher of each kind in a rendered document, and all choices remain
attached to the DOM when the switcher is closed or off-screen.

**`data-value`.** The switcher writer puts this on each machine-readable choice below a
`data-switcher` root. Its nonempty value is the stable page or theme ID, values are unique within
that switcher, and DOM order is the switcher's presentation order. Generic browser helpers read the
values rather than labels, which may change or be hidden. Uses of `data-value` outside a switcher,
such as a Component's own public attribute, have no meaning under this contract.

**`data-baseline`.** The theme-switcher writer puts `data-baseline="true"` on each `data-value`
choice selected for screenshot coverage and omits it from other choices. The generic address sweep
and shell specs read its presence. Marked values are a nonempty subset of the theme switcher's
values; the marker chooses test coverage and does not choose or apply the current theme. The checked
theme control remains the source of the site's theme. Generic helpers pass the theme ID in the
`theme` query parameter and require the matching marked control to be checked after navigation.

Facade protocol writers validate these requirements before rendering. Page entries require
nonempty unique IDs, unique absolute paths, supported kind/navigation/listing/browser-test values,
and nonempty navigation values. Theme manifests require nonempty unique IDs, at least one baseline,
and exactly one initially checked theme. Invalid authored values are programmer errors; the facade
does not emit a partially valid protocol manifest.

Consumer-specific acceptance markers may extend this vocabulary without becoming generic protocol.
Any future generic helper that needs another DOM fact must first add that fact to this versioned
contract rather than depending on Consumer classes or incidental markup.

## The invocation site supplies the adapters

Discovery remains relative to the source file containing each macro invocation. The macros use
`proc_macro::Span::local_file()` to locate that file, resolve `component_root` and Example paths
from it, and emit invocation-relative `include_str!` references. Invocations therefore remain in
the consuming repository. Rust 1.88 is the minimum supported version.

A consuming app supplies these crate-local conventions:

- `crate::example::ExampleSection` is the default `example_section:` path. It accepts
  `documentation: ExampleDocumentation` plus `children: Element`, and owns the Example chrome and
  `data-example` marker. A Consumer may re-export `dioxus_registry_preview::chrome::ExampleSection`
  from this conventional module or provide its own adapter.
- `crate::example::ReadmeSection` is the default `readme_section:` path. It accepts
  `html: &'static str` and owns trusted build-time Markdown presentation. Either adapter path may
  be overridden in a `component!` invocation; the facade exports a default adapter for re-export.
- `component_pages!` emits `example_modules`. The app re-exports it at the crate root with
  `pub(crate) use pages::example_modules as examples;`, or the equivalent for its catalog module,
  so one Example can import another Component's fixture through
  `crate::examples::<component>::<example>` without a second module list.

The adapter selection, catalog, custom pages, and macro invocations are Consumer source. The facade
provides default adapters and a shell without reaching back into a Consumer Registry.

## Default chrome is dependency-owned and isolated

The facade owns a complete default site plus default installation, Example, README, source-block,
theme-switcher, navigation, and shell presentation. `chrome::App` accepts a Consumer's static page catalog and uses
the macro-generated `AppCatalog` wrapper and Consumer-owned `AppPage` implementation to obtain stable
group IDs, rendering-kind names, and rendering. `AppCatalog` preserves the authored group ID for each
generated descriptor from the same complete ID-to-value mapping that resolves Component groups; it
does not reverse that mapping for generated pages, so distinct IDs may share one group value. A
handwritten grouped descriptor carries only the group value and therefore resolves to the first
matching ID in mapping order. Consumers do not maintain a reverse mapping. `App` supplies dynamic
catalog routing, document titles, a not-found page, branding, the default `Sidebar` and theme
switcher, and all protocol writers. `chrome::Shell` remains the smaller composition for existing and
customized Consumers rather than growing App's visible header and navigation.

`component_pages!` resolves Registry installation facts from its root manifest, default Component,
optional source fields, and the invoking package's Cargo metadata. `App` prepends a facade-owned
installation page when no authored descriptor claims `/installation`. Its stable protocol metadata
is ID `registry-installation`, kind `rsx`, standalone navigation under `Getting started`, listed
navigation policy, and enabled browser-test policy. It renders at `/installation` from the Registry
facts attached to `APP_CATALOG`, but is not inserted into `PAGE_CATALOG`. An authored descriptor at
that path suppresses synthesis. The complete decision is recorded in
[ADR 0003](0003-generated-installation-page.md).

When the authored descriptor slice has no exact `/` path, `App` prepends a facade-owned home to its
runtime catalog and renders it at `/`. Its stable protocol metadata is ID `registry-home`, kind
`rsx`, standalone navigation under `Getting started`, unlisted navigation policy, and enabled
browser-test policy. The page names the Registry, includes the root manifest description when
present, and indexes every listed runtime page with standalone sections before Component groups.
Its getting-started action prefers `/installation`, including an authored replacement, and otherwise
targets the first runtime descriptor. An authored `/`
descriptor suppresses synthesis and renders normally, while the brand always links to `/`.
`registry-home` is reserved when this default tier supplies the home; it is not inserted into
`PAGE_CATALOG`, so authored catalog order, validation, and lower-level consumers are unchanged.

The default chrome's Rust source uses only namespaced `rpv-*` classes backed by CSS packaged in
the same crate. `chrome::Shell` mounts that stylesheet and the default light/dark switcher;
`chrome::Styles` makes the stylesheet available below a Consumer-owned shell.

All default selectors are scoped below a `data-registry-preview-chrome` root. The stylesheet contains
no global reset, `:root` selector, Tailwind output, daisyUI classes, or component-library variables.
It therefore cannot restyle the surrounding documentation site or supply the styles of a rendered
Component. Bare element selectors appear only below `.rpv-readme`, whose descendants are the
facade's own build-time Markdown rendering rather than Consumer content. Consumers remain free to
replace the adapters and shell through ordinary Rust source.

The zero-configuration `chrome::App` tier is the whole site, so it additionally owns the document
surface: `App` mounts a second, document-level stylesheet addressing only the `html` and `body`
background, margin, and scrollbar colors so the selected theme covers the full page instead of a
themed island on the user agent's default canvas. `chrome::Shell`, `chrome::Styles`, and the scoped
chrome stylesheet remain isolated below the chrome root for Consumers that own their shell and
surrounding page.

The fixture has no stylesheet of its own; its browser tests prove the packaged chrome CSS applies
through a renamed facade dependency.

## Discovery and diagnostics have a versioned config

> Superseded by [ADR 0002](0002-catalog-invocation-is-site-configuration.md).

Standalone discovery starts from `registry-docs.json`, whose `schemaVersion` is checked before any
Registry input is loaded. Version 1 names the root Registry manifest, the ordered stable group IDs,
and the default Component module. An unsupported version fails immediately with `RDOC001`; this is
the explicit cross-repository skew signal rather than an attempt to interpret a newer shape.

`dioxus-registry-preview-core` owns the ordinary, Dioxus-free model and all generic discovery and validation.
The facade re-exports that validation interface for Consumer tests. Diagnostics retain stable
`RDOC###` identifiers and full-site loading returns every independent diagnostic in one run. The
proc macros adapt diagnostics back to invocation spans, while Registry acceptance policy remains
ordinary Consumer test code and does not enter the generic model.

## Deferred decisions

This contract deliberately does not decide how Component props and public API become reference
documentation, or how search, sitemap,
and link-checking artifacts are emitted. Those features may deepen the metadata and catalog later
without changing generated section names, stable group IDs, invocation conventions, or marker
contract version 1.
