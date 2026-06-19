/**
 * Repository for the `templates` table (frontend).
 *
 * Rule 3 — the `json_schema` column always holds a serialized `OmrTemplate`.
 * Backdrop image path lives in the sidecar `backdrop_path` column (added in
 * migration 0002) so it never contaminates the round-trip JSON.
 *
 * Every read passes the parsed JSON through `templateSchema.parse` so a
 * silently-corrupt row surfaces as a typed error rather than leaking garbage
 * into the editor.
 */
import { getDb } from "./index";
import {
  templateSchema,
  type OmrTemplate,
  type TemplateSummary,
} from "$lib/types/template";

interface TemplateRow {
  id: number;
  title: string;
  json_schema: string;
  backdrop_path: string | null;
}

function rowToSummary(r: TemplateRow): TemplateSummary {
  const parsed = JSON.parse(r.json_schema) as unknown;
  const validated = templateSchema.parse(parsed) as OmrTemplate;
  return {
    id: r.id,
    title: r.title,
    schema: validated,
    backdropPath: r.backdrop_path ?? undefined,
  };
}

export async function listTemplates(): Promise<TemplateSummary[]> {
  const db = await getDb();
  const rows = await db.select<TemplateRow[]>(
    "SELECT id, title, json_schema, backdrop_path FROM templates ORDER BY id DESC",
  );
  return rows.map(rowToSummary);
}

export async function getTemplate(id: number): Promise<TemplateSummary | null> {
  const db = await getDb();
  const rows = await db.select<TemplateRow[]>(
    "SELECT id, title, json_schema, backdrop_path FROM templates WHERE id = $1",
    [id],
  );
  return rows.length > 0 ? rowToSummary(rows[0]!) : null;
}

export async function createTemplate(
  template: OmrTemplate,
  backdropPath: string | null,
): Promise<number> {
  const db = await getDb();
  const json = JSON.stringify(template);
  const result = await db.execute(
    "INSERT INTO templates (title, json_schema, backdrop_path) VALUES ($1, $2, $3)",
    [template.title, json, backdropPath],
  );
  return Number(result.lastInsertId);
}

export async function updateTemplate(
  id: number,
  template: OmrTemplate,
  backdropPath: string | null,
): Promise<void> {
  const db = await getDb();
  const json = JSON.stringify(template);
  await db.execute(
    "UPDATE templates SET title = $1, json_schema = $2, backdrop_path = $3 WHERE id = $4",
    [template.title, json, backdropPath, id],
  );
}

export async function deleteTemplate(id: number): Promise<void> {
  const db = await getDb();
  await db.execute("DELETE FROM templates WHERE id = $1", [id]);
}
