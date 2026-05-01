//! 학생 도메인 모델 — `students` 테이블의 행과 1:1 대응.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student {
    pub id: i64,
    pub name: String,
    pub grade: i32,
    pub class: i32,
    pub roll_number: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewStudent {
    pub name: String,
    pub grade: i32,
    pub class: i32,
    pub roll_number: String,
}
