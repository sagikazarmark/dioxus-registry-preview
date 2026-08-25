import type { Locator, Page } from "@playwright/test";

export type CatalogPage = {
  id: string;
  path: string;
};

export type DocumentationAddress = {
  page?: CatalogPage;
  theme?: string;
};

export type Motion = "still" | "running";

export function openPreview(
  page: Page,
  address?: DocumentationAddress,
  motion?: Motion,
): Promise<void>;

export function addresses(page: Page): Promise<{ page: CatalogPage; theme: string }[]>;

export function example(page: Page, slug: string): Locator;

export function themes(page: Page): Promise<string[]>;

export function computedStyle(
  elements: Locator,
  property: string,
  pseudo?: string,
): Promise<string[]>;
