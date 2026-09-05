# Default Chrome

The `dioxus-registry-preview` facade owns a complete default site and presentation for installation instructions, Examples, READMEs, source blocks, navigation, themes, and the site shell. Its stylesheets are compiled into the facade. `chrome::App` turns a page catalog into the complete site, `chrome::Shell` mounts the default stylesheet and light/dark switcher, `chrome::Sidebar` mounts its navigation stylesheet, and `chrome::Styles` can mount the default content styles under a Consumer-owned shell.

Default chrome uses `rpv-*` classes beneath one `data-registry-preview-chrome` root. Its CSS contains no global reset, root selector, Tailwind utilities, or component-library theme variables, so a Consumer's stylesheet continues to own the surrounding page and rendered Components. Bare element selectors appear only below `.rpv-readme`, whose descendants are the facade's own build-time Markdown rendering; Example content and every other Consumer-rendered element stay untouched. With the `syntax-highlighting` feature, highlighted code additionally mounts `dioxus-code`'s class-scoped `.dxc*` and `.a-*` stylesheets, which likewise select no bare element or root.

The zero-configuration `App` is the whole site, so it also owns the document surface: `App` mounts an additional document-level stylesheet that sets only the `html` and `body` background, margin, and scrollbar colors, making the selected theme cover the full page. `Shell` and `Styles` never mount that stylesheet — a Consumer with its own shell keeps full ownership of the document.

The facade exports:

- `chrome::App`, the batteries-included dynamic router, generated installation and home pages, titles, not-found page, header, navigation, themes, shell, and protocol writers;
- `chrome::AppCatalog`, the generated descriptor, group-ID, and Registry-facts input to `App`, and `chrome::AppPage`, the Consumer-owned seam for rendering-kind names and literal page rendering;
- `chrome::RegistryDocumentation`, `chrome::RegistrySource`, and `chrome::InstallationPage`, the reusable installation model and default presentation;
- `chrome::ExampleSection` and `chrome::ReadmeSection`, including protocol markers;
- `chrome::CodeBlock` and `chrome::CodeContent`, code presentation without a rendered preview, used by the Code tab and the installation page;
- `chrome::PageCatalogManifest` and `chrome::ThemeSwitcherManifest`, the protocol writers a Consumer-owned shell mounts directly;
- `chrome::ThemeSwitcher`, the visible default light/dark control and its theme manifest;
- `chrome::Sidebar`, catalog-driven visible navigation with a Consumer-provided link builder;
- `chrome::Shell`, the default composition of the stylesheet, page manifest, theme switcher, and current-page wrapper; and
- `chrome::Styles`, for Consumers that retain their own shell.

Consumers mount protocol writers; they do not copy protocol-shaped markup into their shells. The conventional `crate::example` module may re-export the facade's Example and README adapters or define Consumer-owned replacements. No command copies or updates source files. Browser acceptance proves the packaged CSS applies without a Consumer stylesheet.

## Examples and code

`ExampleSection` shows the Example title and description beside a Preview / Code tab pair. Preview contains the rendered Example and the `data-example-content` marker and is selected initially; Code contains the exact Rust file the Example compiled from. Both panels stay in the DOM and the inactive one is hidden, so switching tabs never remounts the Example. The tabs follow the WAI-ARIA tabs pattern: roving focus, arrow keys, Home, and End.

`CodeBlock` shows code without a preview. It accepts `CodeContent`, into which `&'static str`, `String`, and `Cow<'static, str>` convert as plain text, and an optional `label` that captions the panel with a file name or language. The installation page uses it for shell commands and `Dioxus.toml`; a Consumer page can show a manifest with `CodeBlock { content: include_str!("component.json"), label: "component.json" }`.

With the facade's `syntax-highlighting` feature, `component!` highlights every Example source at compile time with tree-sitter's Rust grammar and the Code tab renders it through `dioxus_code::Code` with a fixed dark token theme; the chrome supplies the panel surface and typography so plain and highlighted blocks match in both chrome themes. `ExampleDocumentation::code()` returns that highlighted source, and `CodeBlock` accepts any `dioxus_code::advanced::HighlightedSource`, including a Consumer's own `dioxus_code::code!` or `code_str!` output. Those macros locate `dioxus-code` in the Consumer's own `Cargo.toml`, so a Consumer using them depends on `dioxus-code` directly; non-Rust languages there are feature-gated and pull in `dioxus-code`'s runtime parser. The feature needs a C compiler that targets `wasm32-unknown-unknown`; see [compatibility](compatibility.md). Without it, the Code tab and `CodeBlock` render plain preformatted text and the build compiles no C.

## Default site

`App` accepts the static `APP_CATALOG` emitted by `component_pages!` and an optional branding title, which defaults to `"Registry documentation"`. `APP_CATALOG` pairs the existing `PAGE_CATALOG` descriptor slice with each generated page's authoritative authored group ID, root Registry description, and installation facts without adding `App` trait bounds to consumers of the lower-level slice. Distinct stable IDs may map to the same group value without generated pages losing their identity. A handwritten grouped page has only a group value, so it uses the first matching ID in mapping order. `App` converts the descriptors into the chrome view model, routes over their exact absolute paths, preserves the current query string in navigation, and writes each selected descriptor's title to the document. An unknown non-root path renders a titled not-found page without claiming a `data-page` ID.

`component_pages!` attaches the Registry name, description, ordered Component install names, resolved default install name, repository URL, and optional revision to `APP_CATALOG`. The repository defaults to the invoking package's `CARGO_PKG_REPOSITORY`; `repository:` overrides it and `revision:` supplies an explicit Git pin. `App` uses those facts to prepend `registry-installation` at `/installation` when no authored descriptor claims that path. The entry is `rsx`, listed under `Getting started`, and browser-test enabled. The page shows `dx components add` for the default Component and `--all`, plus persistent `Dioxus.toml` configuration and the ordered install-name list.

When no authored descriptor has `path == "/"`, `App` prepends a facade-owned entry to its runtime catalog and renders a default home at `/`. The entry has ID `registry-home`, kind `rsx`, standalone navigation under `Getting started`, unlisted navigation policy, and enabled browser-test policy. It appears in `data-page-catalog` and supplies the root `data-page`, but it is not inserted into the Consumer's generated `PAGE_CATALOG`. The home names the Registry, shows the root manifest description when nonempty, and links every listed runtime page as standalone sections followed by Component groups. Its Getting started action prefers `/installation` and otherwise links to the first runtime descriptor.

Authored descriptors at `/installation` or `/` suppress the corresponding synthesis and win without another option. The default brand link always targets `/`. When synthesis is needed, `chrome::App` reserves `registry-installation` or `registry-home`; similarly named authored IDs remain valid. Generated entries appear only in `App`'s runtime catalog, not `PAGE_CATALOG`. A custom shell may render `InstallationPage` from `APP_CATALOG.registry()` and supply its own descriptor.

The Consumer's rendering kind remains in control. Its `AppPage::render` implementation dispatches generated variants through `ComponentPage::view`, whose macro-generated match retains literal `DocumentationPage` calls, and may render handwritten variants directly. `App` does not turn Component rendering into data or weaken the same-file guarantee.

`App` uses a facade-private catch-all `Routable` route to select catalog entries dynamically. That zero-configuration tier is separate from a customized Consumer site whose public route model remains a static `Routable` enum.

## Default themes

`ThemeSwitcher` offers `light` and `dark`, both selected for baseline browser coverage. On startup it uses a valid `?theme=` address or falls back to `prefers-color-scheme`. It writes the selected value to `data-theme` on the document element and keeps the visible controls, hidden `ThemeSwitcherManifest`, and current URL synchronized. The default stylesheet responds with isolated light/dark values only below the chrome root; a Consumer stylesheet continues to own the rest of the page.

`Shell` mounts `ThemeSwitcher` when a Consumer uses the default composition. A Consumer with its own shell omits that component and mounts `ThemeSwitcherManifest` beside its own visible control, feeding both from the same theme model. This is the override path for an N-theme dropdown such as the daisyUI Registry's switcher: its `ThemeController` controls page theming through the document-root `:has()` mechanism, while the manifest continues to expose every theme to generic browser helpers.
