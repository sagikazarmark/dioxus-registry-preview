# Browser Helpers

`@sagikazarmark/dioxus-registry-preview-playwright` consumes DOM protocol v1 without depending on Consumer classes or labels.

- `openPreview(page, address, motion)` opens a catalog path, writes the theme as the `theme` query parameter, waits for the catalog and current-page markers, verifies the matching theme control is checked, and optionally quiesces motion.
- `addresses(page)` returns every browser-enabled catalog page under every baseline theme, including pages omitted from navigation.
- `example(page, slug)` returns the marked rendered content of one Example.
- `themes(page)` returns stable theme IDs in switcher order.
- `computedStyle(locator, property, pseudo?)` reads one computed property from every matched element.

The package is plain JavaScript with TypeScript declarations. Consumers can pin the Git repository revision in `package.json`; npm records the resolved commit in its lockfile.

Consumer-specific acceptance vocabulary belongs beside the Consumer tests. For example, Axis markers and assertions about a CSS component library do not belong in these generic helpers.
