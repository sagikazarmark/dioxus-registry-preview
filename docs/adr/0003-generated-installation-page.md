# The default App generates installation from Registry facts

**Status:** accepted.

Every Registry Preview needs installation instructions, while the root Registry manifest,
Component manifests, and `component_pages!` invocation already provide the Registry name,
description, ordered install names, and default Component. The remaining source is the Registry's
Git repository. `component_pages!` therefore accepts optional `repository:` and `revision:` string
literals. The repository defaults to the invoking package's `CARGO_PKG_REPOSITORY`; the revision is
always explicit because package versions and Git refs are separate release-policy decisions.

The macro validates the resolved source and emits all installation facts into `APP_CATALOG` as
`chrome::RegistryDocumentation`. A missing repository is `RDOC037`, a repository that is not an
absolute `git`, `http`, `https`, or `ssh` URL is `RDOC038`, and an empty, whitespace-containing, or
option-shaped revision is `RDOC039`. An override supports a Preview package whose Cargo repository
does not identify the Registry repository. A Registry in a subdirectory of a monorepo still needs a
repository whose root contains `component.json`, because `dx components add --git` clones and reads
the repository root. The root manifest name is now required and whitespace-only names produce
`RDOC040`, preventing a generated page with an empty Registry heading.

`chrome::App` uses these facts to prepend a facade-owned runtime entry when no authored descriptor
claims `/installation`. Its stable metadata is ID `registry-installation`, path `/installation`, kind
`rsx`, standalone navigation under `Getting started`, listed navigation policy, and enabled browser
testing. The page shows a worked command for the default Component's manifest install name, an
`--all` command, persistent `Dioxus.toml` configuration, and every Component install name in root
manifest order. A configured revision appears in both command and configuration forms. Commands
shell-quote source values, and the configuration escapes TOML strings.

The generated entry appears in `App`'s runtime DOM catalog, routing, navigation, browser sweep, and
default-home index, but not in the authored `PAGE_CATALOG` slice. This preserves the Consumer-owned
rendering kind and keeps lower-level catalog users free of a facade page variant. An authored
`/installation` descriptor suppresses synthesis. While synthesis is active, `registry-installation`
is reserved. Custom shells may render the exported `chrome::InstallationPage` from
`APP_CATALOG.registry()` and author their own descriptor.

Per-Component install sections remain deferred. Independent `component!` invocations intentionally
have no site-level repository context, and adding that context would broaden the generated Component
page contract for duplicate prose already covered by the Registry installation page.
