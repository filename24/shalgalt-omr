/**
 * Repository for the `templates` table (frontend).
 *
 * Rule 3 — the `json_schema` column always holds a serialized `OmrTemplate`.
 */
import { getDb } from "./index";
import type { OmrTemplate, TemplateSummary } from "$lib/types/template";

interface TemplateRow {
  id: number;
  title: string;
  json_schema: string;
}

function rowToSummary(r: TemplateRow): TemplateSummary {
  return {
    id: r.id,
    title: r.title,
    schema: JSON.parse(r.json_schema) as OmrTemplate,
  };
}

export async function listTemplates(): Promise<TemplateSummary[]> {
  const db = await getDb();
  const rows = await db.select<TemplateRow[]>(
    "SELECT id, title, json_schema FROM templates ORDER BY id DESC",
  );
  return rows.map(rowToSummary);
}

export async function getTemplate(id: number): Promise<TemplateSummary | null> {
  const db = await getDb();
  const rows = await db.select<TemplateRow[]>(
    "SELECT id, title, json_schema FROM templates WHERE id = $1",
    [id],
  );
  return rows.length > 0 ? rowToSummary(rows[0]!) : null;
}

export async function createTemplate(template: OmrTemplate): Promise<number> {
  const db = await getDb();
  const json = JSON.stringify(template);
  const result = await db.execute(
    "INSERT INTO templates (title, json_schema) VALUES ($1, $2)",
    [template.title, json],
  );
  return Number(result.lastInsertId);
}

export async function updateTemplate(
  id: number,
  template: OmrTemplate,
): Promise<void> {
  const db = await getDb();
  const json = JSON.stringify(template);
  await db.execute(
    "UPDATE templates SET title = $1, json_schema = $2 WHERE id = $3",
    [template.title, json, id],
  );
}

export async function deleteTemplate(id: number): Promise<void> {
  const db = await getDb();
  await db.execute("DELETE FROM templates WHERE id = $1", [id]);
}
