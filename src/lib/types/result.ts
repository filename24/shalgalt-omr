/** Mirrors Rust `commands::results::ResultSummary`. */
export interface ResultSummary {
  id: number;
  student_id: number | null;
  template_id: number;
  total_score: number;
  detail_answers_json: string;
  image_path: string | null;
}
