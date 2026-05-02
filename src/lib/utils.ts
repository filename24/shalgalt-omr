import { type ClassValue, clsx } from "clsx";
import { twMerge } from "tailwind-merge";

/**
 * Tailwind class merge helper required by every shadcn-svelte component.
 *
 * Flattens conditional/array inputs via `clsx`, then resolves conflicting
 * Tailwind utilities via `tailwind-merge` (e.g. `px-2 px-4` → `px-4`).
 */
export function cn(...inputs: ClassValue[]): string {
  return twMerge(clsx(inputs));
}

/**
 * bits-ui helper types re-exported for shadcn-svelte components.
 * The CLI-generated `.svelte` files import these directly from `$lib/utils`,
 * so they must live here even though they originate in bits-ui.
 */
export type WithElementRef<T, U extends HTMLElement = HTMLElement> = T & {
  ref?: U | null;
};

export type WithoutChild<T> = T extends { child?: unknown } ? Omit<T, "child"> : T;

export type WithoutChildren<T> = T extends { children?: unknown }
  ? Omit<T, "children">
  : T;

export type WithoutChildrenOrChild<T> = WithoutChildren<WithoutChild<T>>;
