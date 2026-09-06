# Code presentation is compile-time highlighted behind a feature

**Status:** accepted.

The default Example chrome shows one rendered Example and the exact file it was compiled from.
Presenting that file as highlighted Rust, and presenting other code such as manifests without a
rendered preview, is presentation owned by the facade. It does not change the DOM protocol: the
`data-example` root and the `data-example-content` pane keep their promises, and tabs, panels, and
code markup remain outside the versioned marker vocabulary.

## Example sections have Preview and Code tabs

`chrome::ExampleSection` renders the Example title and description beside a `tablist` with two
tabs, `Preview` and `Code`. Both panels stay attached to the DOM so switching never remounts the
Example or loses its state; the inactive panel carries the `hidden` attribute. `Preview` is
selected initially, so generic Example lookup finds a visible `data-example-content` pane without
interacting with the chrome. The tabs implement the WAI-ARIA tabs pattern with roving focus and
arrow, Home, and End keys. Tab and panel `id`s derive from the Example slug, which is unique
within a page.

The Code tab contains Rust: the file named by the Example module, included verbatim by
`include_str!`. Its label is `Code` rather than `RSX` because the file is an ordinary Rust module,
not only an `rsx!` body.

## Highlighting runs when the module is compiled

Example sources are static files, so `component!` highlights them at expansion time with
`arborium`'s Rust grammar and emits the resulting spans as a `&'static [HighlightSpan]` next to
the included source. `ExampleDocumentation::highlights` carries those spans and
`ExampleDocumentation::code()` pairs them with `source` as a `dioxus_code::advanced::HighlightedSource`.
The same-file guarantee holds: the highlighted text is the one `include_str!` expression that the
Example compiles from. No parser ships to the browser and no parsing happens at runtime.

The facade renders highlighted source through `dioxus_code::Code` with one fixed dark token
theme. The code panel is deliberately dark under both chrome themes, so the chrome stylesheet
overrides the theme's background with its own `--rpv-code-surface` and typography, and plain and
highlighted blocks share one surface. Token colors are the only contribution of the theme.

## The `syntax-highlighting` feature is opt-in

`dioxus-code` depends on `arborium`, whose tree-sitter runtime is C compiled by `cc` for every
target, including `wasm32-unknown-unknown`, regardless of enabled features. Depending on it
therefore requires a C compiler that targets WebAssembly on every machine that builds a Preview.
Apple's bundled clang does not; Homebrew or Nix LLVM with `CC_wasm32_unknown_unknown` set does.

The facade keeps that requirement out of the default build. The `syntax-highlighting` feature
adds `dioxus-code` to the facade and forwards to a same-named feature on the macros package, which
alone adds `arborium` to the proc-macro build. Without the feature no crate in the graph compiles
C, the macro emits the four-argument `ExampleDocumentation::new`, and the Code tab renders the
same source as plain preformatted text. `ExampleDocumentation::highlights`, `code()`, the
`code` module, and `CodeContent::Highlighted` exist only with the feature. Enabling the feature
in a compatible release stays compatible; making it a default would be a new build requirement
and needs at least a minor release before 1.0.

## Standalone code has no preview

`chrome::CodeBlock` presents code without a rendered Example. It accepts `CodeContent`: plain
text always, which the generated installation page uses for shell commands and `Dioxus.toml`, and
`HighlightedSource` with the feature. An optional label captions the panel with a file name or
language. Consumers can therefore show a manifest with `include_str!("component.json")` in any
build, or pass their own compile-time snippet from `dioxus_code::code!` or `code_str!`. Those
macros resolve `dioxus-code` in the Consumer's own manifest, so a Consumer using them adds
`dioxus-code` as a direct dependency; and every non-Rust `Language` variant in `dioxus-code`
0.1 is gated behind a `lang-*` feature that also enables its runtime parser, so highlighted JSON is
never free. The facade does not paper over that: plain JSON is the zero-cost path and highlighted
JSON is the Consumer's explicit choice.

## Isolated chrome, amended

The default chrome stylesheet remains scoped below `data-registry-preview-chrome` with `rpv-*`
classes only. With the feature, `dioxus_code::Code` additionally mounts its own class-scoped
stylesheets: `.dxc` base rules, one `.dxc-<theme>` variable block, and `.dxc .a-*` token colors.
They select no bare element, root, or reset, so Consumer content stays untouched, but they are not
below the chrome root. The chrome's own rules for `.rpv-code-panel .dxc` override the base
stylesheet's layout and background. README code fences are unchanged and remain a possible later
use of core's `MarkdownHooks::code_block` seam.
