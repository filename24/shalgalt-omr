/**
 * `students` 테이블 Repository (프론트엔드).
 *
 * P3에서 본격 사용. P0 단계에서는 list/insert만 제공.
 */
import { getDb } from "./index";

export interface Student {
  id: number;
  name: string;
  grade: number;
  class: number;
  roll_number: string;
}

export interface NewStudent {
  name: string;
  grade: number;
  class: number;
  roll_number: string;
}

export async function listStudents(): Promise<Student[]> {
  const db = await getDb();
  return db.select<Student[]>(
    `SELECT id, name, grade, class, roll_number FROM students
     ORDER BY grade, class, roll_number`,
  );
}

export async function insertStudent(s: NewStudent): Promise<number> {
  const db = await getDb();
  const result = await db.execute(
    `INSERT INTO students (name, grade, class, roll_number) VALUES ($1, $2, $3, $4)`,
    [s.name, s.grade, s.class, s.roll_number],
  );
  return Number(result.lastInsertId);
}
