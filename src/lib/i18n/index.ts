/**
 * Mongolian-only i18n entry point.
 *
 * Re-exports the string table and provides a strongly-typed dotted-path lookup
 * (`t`) for cases where the path is dynamic (e.g. driven by an enum value).
 *
 * For static usage, components should prefer direct nested access:
 *   `mn.editor.toolbar.save`
 * which gives full autocomplete and compile-time miss detection without the
 * lookup runtime.
 */
import { mn } from "./mn";

export { mn };

type Primitive = string | number | boolean | null | undefined;

/**
 * All dotted paths into a nested string table that resolve to a string leaf.
 * Compile-time miss detection: a typo or removed key fails to typecheck.
 */
type LeafPaths<T, Prefix extends string = ""> = {
  [K in keyof T & string]: T[K] extends Primitive
    ? T[K] extends string
      ? `${Prefix}${K}`
      : never
    : LeafPaths<T[K], `${Prefix}${K}.`>;
}[keyof T & string];

export type StringPath = LeafPaths<typeof mn>;

/**
 * Resolve a dotted path into the `mn` table. Throws on a missing key — this
 * should never happen at runtime because the type system rules it out, but the
 * throw exists so a structural change to `mn` cannot silently leak `undefined`
 * into a rendered string.
 */
export function t(path: StringPath): string {
  const parts = path.split(".");
  let cursor: unknown = mn;
  for (const part of parts) {
    if (cursor && typeof cursor === "object" && part in cursor) {
      cursor = (cursor as Record<string, unknown>)[part];
    } else {
      throw new Error(`Missing string-table key: ${path}`);
    }
  }
  if (typeof cursor !== "string") {
    throw new Error(`String-table key ${path} did not resolve to a string`);
  }
  return cursor;
}
