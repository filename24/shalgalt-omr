/**
 * Repository for the `students` table (frontend).
 *
 * Used in earnest from P3 onward. P0 only exposes list / insert.
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
