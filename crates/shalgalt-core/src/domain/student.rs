//! Student domain model — 1:1 with rows of the `students` table.

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct Student {
    /// SQLite `INTEGER PRIMARY KEY` — fits in JS `number` for any realistic
    /// roster size, so the generated TS uses `number` instead of `bigint`.
    #[ts(type = "number")]
    pub id: i64,
    pub name: String,
    pub grade: i32,
    pub class: i32,
    pub roll_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../../src/lib/types/generated/")]
pub struct NewStudent {
    pub name: String,
    pub grade: i32,
    pub class: i32,
    pub roll_number: String,
}
