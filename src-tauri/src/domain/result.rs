//! 채점 결과 도메인 모델.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GradedAnswer {
    /// 단일 마킹 — 정답.
    Correct {
        group_id: String,
        marked_index: u32,
    },
    /// 단일 마킹 — 오답.
    Wrong {
        group_id: String,
        marked_index: u32,
        correct_index: u32,
    },
    /// 마킹 없음.
    Blank { group_id: String },
    /// 다중 마킹.
    Multiple {
        group_id: String,
        marked_indices: Vec<u32>,
    },
}

/// 한 장의 OMR 시트에 대한 채점 결과 — `results.detail_answers` 컬럼에 직렬화된다.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GradedSheet {
    pub template_id: i64,
    pub student_id: Option<i64>,
    pub total_score: f32,
    pub answers: Vec<GradedAnswer>,
    /// asset:// 으로 노출 가능한 결과 이미지 절대경로.
    pub image_path: Option<String>,
}
