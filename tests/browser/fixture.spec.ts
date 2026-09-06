import { expect, test } from "@playwright/test";

import {
  addresses,
  computedStyle,
  example,
  openPreview,
  themes,
} from "@sagikazarmark/dioxus-registry-preview-playwright";

const customHomeBaseURL = "http://127.0.0.1:4174";

test("generic helpers consume the fixture's protocol", async ({ page }) => {
  const discovered = await addresses(page);
  expect(discovered).toEqual([
    { page: { id: "registry-home", path: "/" }, theme: "light" },
    { page: { id: "registry-home", path: "/" }, theme: "dark" },
    { page: { id: "registry-installation", path: "/installation" }, theme: "light" },
    { page: { id: "registry-installation", path: "/installation" }, theme: "dark" },
    { page: { id: "default_component", path: "/default_component" }, theme: "light" },
    { page: { id: "default_component", path: "/default_component" }, theme: "dark" },
    { page: { id: "custom_page", path: "/custom_page" }, theme: "light" },
    { page: { id: "custom_page", path: "/custom_page" }, theme: "dark" },
    { page: { id: "nested_layout", path: "/nested_layout" }, theme: "light" },
    { page: { id: "nested_layout", path: "/nested_layout" }, theme: "dark" },
  ]);

  for (const address of discovered) {
    await openPreview(page, address);
  }

  const defaultPage = discovered.find(({ page }) => page.id === "default_component");
  expect(defaultPage).toBeDefined();
  await openPreview(page, defaultPage!);
  await expect(example(page, "overview")).toBeVisible();
  await expect(page).toHaveTitle("Default page | Registry documentation");
  expect(await themes(page)).toEqual(["light", "dark"]);
});

test("App synthesizes a styled catalog home at root", async ({ page }) => {
  for (const [theme, surface] of [
    ["light", "rgb(240, 237, 231)"],
    ["dark", "rgb(29, 31, 36)"],
  ] as const) {
    await openPreview(page, {
      page: { id: "registry-home", path: "/" },
      theme,
    });

    await expect(page).toHaveTitle("Home | Registry documentation");
    await expect(page.locator(".rpv-home__title")).toHaveText("Registry documentation");
    await expect(page.locator(".rpv-home__description")).toHaveText(
      "External-consumer fixture for the documentation macros.",
    );
    await expect(page.locator(".rpv-home__entry-title")).toHaveText([
      "Installation",
      "Manual assembly",
      "Default page",
      "Nested layout",
    ]);
    await expect(page.locator(".rpv-home__entry-description")).toHaveText([
      "Install components from this Registry.",
      "A handwritten fixture page.",
      "The generated default assembly.",
      "A fixture with nested Example imports.",
    ]);
    await expect(page.locator(`.rpv-home__link:not([href$="?theme=${theme}"])`)).toHaveCount(0);
    await expect(page.locator(".rpv-home__cta")).toHaveAttribute(
      "href",
      `/installation?theme=${theme}`,
    );
    expect(await computedStyle(page.locator(".rpv-home__link").first(), "background-color")).toEqual([
      surface,
    ]);
  }
});

test("App generates installation instructions from Registry facts", async ({ page }) => {
  await openPreview(page, {
    page: { id: "registry-installation", path: "/installation" },
    theme: "light",
  });

  await expect(page).toHaveTitle("Installation | Registry documentation");
  await expect(page.locator(".rpv-installation__title")).toHaveText(
    "Install docs-registry-fixture",
  );
  await expect(page.locator(".rpv-installation__description")).toHaveText(
    "External-consumer fixture for the documentation macros.",
  );
  await expect(page.locator(".rpv-installation__component")).toHaveText([
    "default_component",
    "custom_page",
    "nested_layout",
  ]);
  await expect(page.locator(".rpv-code")).toContainText([
    "dx components add default_component --git 'https://github.com/sagikazarmark/dioxus-registry-docs'",
    "dx components add --all --git 'https://github.com/sagikazarmark/dioxus-registry-docs'",
    '[components]\nregistry = { git = "https://github.com/sagikazarmark/dioxus-registry-docs" }',
  ]);
  await expect(page.locator(".rpv-installation")).toBeVisible();
  expect(await computedStyle(page.locator(".rpv-installation"), "display")).toEqual(["flex"]);
  await expect(page.locator('[data-catalog-page="registry-installation"]')).toHaveAttribute(
    "data-browser-test",
    "enabled",
  );
});

test("an authored root page replaces the default home", async ({ page }) => {
  await page.goto(`${customHomeBaseURL}/`);

  await expect(page.locator('[data-page="custom-home"]')).toBeVisible();
  await expect(page.getByRole("heading", { name: "Consumer-owned home" })).toBeVisible();
  await expect(page.locator('[data-catalog-page="registry-home"]')).toHaveCount(0);
  await expect(page.locator('[data-catalog-page="custom-home"]')).toHaveAttribute("data-path", "/");
  await expect(page).toHaveTitle("Custom home | Custom Home Registry");
});

test("openPreview survives default theme canonicalization", async ({ page }) => {
  await openPreview(page);

  await expect(page).toHaveURL(/\?theme=(light|dark)$/);
});

test("default theme switcher follows the address and user choice", async ({ page }) => {
  await page.goto("/default_component?theme=dark");

  const switcher = page.locator(".rpv-theme-switcher");
  const visibleDark = switcher.locator('input[value="dark"]');
  const manifestDark = page.locator('[data-switcher="theme"] [data-value="dark"]');
  await expect(visibleDark).toBeChecked();
  await expect(manifestDark).toBeChecked();
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  await expect(switcher).toBeVisible();
  expect(await computedStyle(switcher, "display")).toEqual(["flex"]);
  expect(await computedStyle(page.locator(".rpv-shell"), "background-color")).toEqual([
    "rgb(18, 19, 22)",
  ]);
  expect(await computedStyle(page.locator("html"), "background-color")).toEqual([
    "rgb(18, 19, 22)",
  ]);
  expect(await computedStyle(example(page, "overview").getByRole("button"), "color-scheme")).toEqual([
    "normal",
  ]);

  await switcher.locator('input[value="light"]').check();

  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
  await expect(page.locator('[data-switcher="theme"] [data-value="light"]')).toBeChecked();
  expect(await computedStyle(page.locator(".rpv-shell"), "background-color")).toEqual([
    "rgb(250, 249, 247)",
  ]);
  expect(await computedStyle(page.locator("html"), "background-color")).toEqual([
    "rgb(250, 249, 247)",
  ]);
  await expect(page).toHaveURL(/\?theme=light$/);

  await page.getByRole("link", { name: "Nested layout" }).click();

  await expect(page).toHaveURL(/\/nested_layout\?theme=light$/);
  await expect(page.locator("html")).toHaveAttribute("data-theme", "light");
});

test("default theme follows the preferred color scheme without an address", async ({ page }) => {
  await page.emulateMedia({ colorScheme: "dark" });
  await page.goto("/default_component");

  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
  await expect(page.locator('.rpv-theme-switcher input[value="dark"]')).toBeChecked();
  await expect(page).toHaveURL(/\?theme=dark$/);

  await page.getByRole("link", { name: "Nested layout" }).click();

  await expect(page).toHaveURL(/\/nested_layout\?theme=dark$/);
  await expect(page.locator("html")).toHaveAttribute("data-theme", "dark");
});

test("Example lookup accepts protocol IDs that need CSS escaping", async ({ page }) => {
  const slug = 'quote"slash\\line\nfeed';
  await page.setContent("<main></main>");
  await page.locator("main").evaluate((main, value) => {
    const root = document.createElement("section");
    root.setAttribute("data-example", value);
    const content = document.createElement("div");
    content.setAttribute("data-example-content", "true");
    root.append(content);
    main.append(root);
  }, slug);

  await expect(example(page, slug)).toHaveCount(1);
});

test("uncataloged paths do not claim a current page ID", async ({ page }) => {
  await page.goto("/missing");
  await expect(page.locator("[data-page]")).toHaveCount(0);
  await expect(page.getByRole("heading", { name: "Page not found" })).toBeVisible();
  await expect(page).toHaveTitle("Page not found | Registry documentation");
});

test("App routes literal encoded catalog paths", async ({ page }) => {
  await page.goto("/exact/a%20b/");

  await expect(page.locator('[data-page="exact-path"]')).toBeVisible();
  await expect(page.getByRole("heading", { name: "Exact catalog path" })).toBeVisible();
  await expect(page.locator('[data-catalog-page="exact-path"]')).toHaveAttribute(
    "data-navigation-value",
    "shared",
  );
  await expect(page.locator('[data-catalog-page="default_component"]')).toHaveAttribute(
    "data-navigation-value",
    "fixtures",
  );
});

test("generated pages keep browser coverage when omitted from navigation", async ({ page }) => {
  await page.goto("/manual-page");

  const custom = page.locator('[data-page-catalog] [data-catalog-page="custom_page"]');
  await expect(custom).toHaveAttribute("data-path", "/custom_page");
  await expect(custom).toHaveAttribute("data-listing", "unlisted");
  await expect(custom).toHaveAttribute("data-browser-test", "enabled");
});

test("App provides catalog-driven navigation", async ({ page }) => {
  await page.goto("/default_component?theme=light");

  const sidebar = page.locator(".rpv-sidebar");
  await expect(page.locator(".rpv-header__brand")).toHaveAttribute(
    "href",
    "/?theme=light",
  );
  await expect(sidebar.locator(".rpv-sidebar__heading")).toHaveText([
    "Getting started",
    "Fixture",
    "fixtures",
  ]);
  await expect(sidebar.locator("a")).toHaveText([
    "Installation",
    "Manual assembly",
    "Default page",
    "Nested layout",
  ]);
  await expect(sidebar.locator('a:not([href$="?theme=light"])')).toHaveCount(0);
  await expect(sidebar.locator('[data-value="default_component"]')).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(sidebar).toHaveAttribute("data-switcher", "component");
  await expect(sidebar.locator("[data-value]")).toHaveCount(2);
  expect(await computedStyle(sidebar, "display")).toEqual(["flex"]);
});

test("App navigation preserves modified-click browser behavior", async ({ page, context }) => {
  await page.goto("/default_component?theme=light");
  const popupPromise = context.waitForEvent("page");

  await page
    .getByRole("link", { name: "Nested layout" })
    .click({ modifiers: ["ControlOrMeta"] });

  const popup = await popupPromise;
  await popup.waitForLoadState();
  await expect(page).toHaveURL(/\/default_component\?theme=light$/);
  await expect(popup).toHaveURL(/\/nested_layout\?theme=light$/);
});

test("facade-owned chrome is isolated and styled without consumer utilities", async ({ page }) => {
  await page.goto("/manual-page");

  const manualPage = page.locator('[data-page="manual-page"]');
  const readme = manualPage.locator("[data-readme]");
  const example = manualPage.locator("[data-example]");

  await expect(page.locator('[data-registry-preview-chrome="true"]')).toHaveCount(1);
  await expect(readme).toHaveClass("rpv-readme");
  await expect(example).toHaveClass("rpv-example");
  await expect(readme).toBeVisible();
  await expect(example).toBeVisible();
  expect(await computedStyle(readme, "display")).toEqual(["flex"]);
  expect(await computedStyle(readme, "flex-direction")).toEqual(["column"]);
  expect(await computedStyle(readme, "gap")).toEqual(["16px"]);
  expect(await computedStyle(example, "display")).toEqual(["flex"]);
  expect(await computedStyle(example, "flex-direction")).toEqual(["column"]);
  expect(await computedStyle(example, "gap")).toEqual(["8px"]);
});

test("custom page composes generated sections in consumer order", async ({ page }) => {
  await openPreview(page, {
    page: { id: "custom_page", path: "/custom_page" },
    theme: "light",
  });

  const customPage = page.locator('[data-page="custom_page"]');
  await expect(customPage).toContainText("Consumer-authored content between generated sections.");
  await expect(customPage.locator("[data-example]")).toHaveCount(2);
  expect(await customPage.locator("[data-example]").evaluateAll((examples) =>
    examples.map((example) => example.getAttribute("data-example")),
  )).toEqual(["details", "overview"]);
  await expect(customPage.locator("[data-readme]")).toHaveCount(0);
});

test("default page renders every generated section", async ({ page }) => {
  await page.goto("/default_component");

  const defaultPage = page.locator('[data-page="default_component"]');
  await expect(defaultPage.locator("[data-readme]")).toBeVisible();
  await expect(defaultPage.locator('[data-example="overview"]')).toBeVisible();
});

test("Example sections switch between the Preview and Code tabs", async ({ page }) => {
  await openPreview(page, {
    page: { id: "default_component", path: "/default_component" },
    theme: "light",
  });

  const section = page.locator('[data-example="overview"]');
  const previewTab = section.getByRole("tab", { name: "Preview" });
  const codeTab = section.getByRole("tab", { name: "Code" });
  const codePanel = section.getByRole("tabpanel", { name: "Code", includeHidden: true });
  const highlighted = codePanel.locator("pre.dxc[data-language='rust']");

  await expect(previewTab).toHaveAttribute("aria-selected", "true");
  await expect(example(page, "overview")).toBeVisible();
  await expect(codePanel).toBeHidden();
  await expect(highlighted).toBeAttached();

  await codeTab.click();

  await expect(codeTab).toHaveAttribute("aria-selected", "true");
  await expect(codeTab).toBeFocused();
  await expect(example(page, "overview")).toBeHidden();
  await expect(codePanel).toBeVisible();
  await expect(highlighted).toContainText('rsx! { button { "Default Example" } }');
  // dioxus-code's theme stylesheet reached the bundle through the facade dependency, and the
  // chrome surface overrides the theme background so plain and highlighted blocks match.
  const keyword = highlighted.locator("span.a-k").first();
  await expect(keyword).toHaveText("use");
  expect(await computedStyle(keyword, "color")).toEqual(["rgb(255, 123, 114)"]);
  expect(await computedStyle(highlighted, "background-color")).toEqual(["rgb(34, 37, 43)"]);

  await codeTab.press("ArrowLeft");

  await expect(previewTab).toHaveAttribute("aria-selected", "true");
  await expect(previewTab).toBeFocused();
  await expect(example(page, "overview")).toBeVisible();
  await expect(codePanel).toBeHidden();
});

test("consumer pages show code without a preview", async ({ page }) => {
  await openPreview(page, {
    page: { id: "custom_page", path: "/custom_page" },
    theme: "dark",
  });

  const customPage = page.locator('[data-page="custom_page"]');
  const panels = customPage.locator(".rpv-code-panel:has(.rpv-code-panel__label)");

  await expect(panels.locator(".rpv-code-panel__label")).toHaveText([
    "component.json",
    "Importing an Example",
  ]);
  await expect(panels.nth(0).locator("pre.rpv-code")).toContainText('"name": "custom_page"');
  await expect(panels.nth(1).locator("pre.dxc[data-language='rust'] span.a-k").first()).toHaveText(
    "use",
  );
  expect(await computedStyle(panels.nth(1).locator("pre.dxc"), "background-color")).toEqual([
    "rgb(13, 15, 18)",
  ]);
});
