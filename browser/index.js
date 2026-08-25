const quiesce = `
*, *::before, *::after {
  animation-delay: 0s !important;
  animation-duration: 0s !important;
  transition-delay: 0s !important;
  transition-duration: 0s !important;
  scroll-behavior: auto !important;
  caret-color: transparent !important;
}
`;

/** Opens one catalog address and waits for its protocol markers to render. */
export async function openPreview(page, address = {}, motion = "still") {
  const path = address.page?.path ?? "/";
  const query = new URLSearchParams();
  if (address.theme !== undefined) query.set("theme", address.theme);

  await navigateToPreview(page, `${path}?${query}`, address.theme);
  await page.locator("[data-page-catalog]").waitFor({ state: "attached" });

  if (address.page !== undefined) {
    await page.locator(attributeEquals("data-page", address.page.id)).waitFor({ state: "attached" });
  }

  if (address.theme !== undefined) {
    await checkedThemeControl(page, address.theme).waitFor({ state: "attached" });
  }

  if (motion === "still") {
    await retryAcrossCanonicalNavigation(page, () => quiescePreview(page));
  }
}

/** Every browser-enabled catalog page under every baseline theme. */
export async function addresses(page) {
  await openPreview(page, {}, "running");

  const themeSwitcher = switcher(page, "theme");
  await themeSwitcher.waitFor({ state: "attached" });
  const baselineThemes = themeSwitcher.locator("[data-value][data-baseline]");
  const themes = await retryAcrossCanonicalNavigation(page, () => dataValues(baselineThemes));
  if (themes.length === 0) {
    throw new Error("the preview marks no theme as a baseline");
  }

  await openPreview(page, { theme: themes[0] }, "running");

  const pages = await page
    .locator(
      '[data-page-catalog] [data-catalog-page][data-path][data-browser-test="enabled"]',
    )
    .evaluateAll((nodes) =>
      nodes.map((node) => ({
        id: node.getAttribute("data-catalog-page") ?? "",
        path: node.getAttribute("data-path") ?? "",
      })),
    );

  if (pages.length === 0) {
    throw new Error("the catalog enables no page for browser tests");
  }

  return pages.flatMap((catalogPage) => themes.map((theme) => ({ page: catalogPage, theme })));
}

/** The rendered content of one marked Example. */
export function example(page, slug) {
  return page.locator(`${attributeEquals("data-example", slug)} [data-example-content]`);
}

/** Every theme the documentation site offers. */
export function themes(page) {
  return dataValues(switcher(page, "theme").locator("[data-value]"));
}

/** One computed style property read from every matched element. */
export function computedStyle(elements, property, pseudo) {
  return elements.evaluateAll(
    (nodes, { property, pseudo }) =>
      nodes.map((node) => getComputedStyle(node, pseudo).getPropertyValue(property)),
    { property, pseudo },
  );
}

function switcher(page, kind) {
  return page.locator(`[data-switcher="${kind}"]`);
}

function checkedThemeControl(page, theme) {
  return switcher(page, "theme").locator(`${attributeEquals("data-value", theme)}:checked`);
}

function dataValues(marked) {
  return marked.evaluateAll((nodes) => nodes.map((node) => node.getAttribute("data-value") ?? ""));
}

async function retryAcrossCanonicalNavigation(page, operation) {
  try {
    return await operation();
  } catch (error) {
    if (!(error instanceof Error) || !error.message.includes("Execution context was destroyed")) {
      throw error;
    }
    await page.locator("[data-page-catalog]").waitFor({ state: "attached" });
    await switcher(page, "theme").waitFor({ state: "attached" });
    return operation();
  }
}

async function quiescePreview(page) {
  await page.addStyleTag({ content: quiesce });
  await page.evaluate(
    () =>
      new Promise((resolve) => {
        requestAnimationFrame(() => requestAnimationFrame(() => resolve()));
      }),
  );
  await page.evaluate(async () => {
    await Promise.all(
      document.getAnimations().map(async (animation) => {
        try {
          await animation.finished;
        } catch {
          // A superseded transition rejects `finished`; it is settled too.
        }
      }),
    );
  });
}

async function navigateToPreview(page, url, theme) {
  for (let attempt = 0; attempt < 2; attempt += 1) {
    try {
      await page.goto(url);
    } catch (error) {
      if (theme === undefined || attempt === 1 || !isNavigationInterruption(error)) throw error;
      continue;
    }

    if (theme === undefined || new URL(page.url()).searchParams.get("theme") === theme) return;
  }

  throw new Error(`preview did not retain requested theme ${theme}`);
}

function isNavigationInterruption(error) {
  return (
    error instanceof Error &&
    (error.message.includes("interrupted by another navigation") ||
      error.message.includes("net::ERR_ABORTED"))
  );
}

function attributeEquals(name, value) {
  const escaped = Array.from(value, (character) => {
    const codePoint = character.codePointAt(0);
    return codePoint < 0x20 || codePoint === 0x7f || character === '"' || character === "\\"
      ? `\\${codePoint.toString(16)} `
      : character;
  }).join("");
  return `[${name}="${escaped}"]`;
}
