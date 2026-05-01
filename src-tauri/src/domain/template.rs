//! OMR 템플릿 도메인 모델 — Rule 3에 정의된 직렬화 단위.
//!
//! 좌표는 모두 **템플릿 캔버스의 정규화 좌표 (0.0 ~ 1.0)** 를 기준으로 한다.
//! 실제 스캔 이미지 픽셀 좌표는 `scan::perspective::warp` 단계에서 환산된다.
//! 이 정책 덕분에 동일 템플릿이 다른 DPI / 해상도의 PDF에 그대로 재사용된다.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct TemplatePoint {
    pub x: f32,
    pub y: f32,
}

/// 4-코너 perspective transform 기준점.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marker {
    pub id: String,
    pub position: TemplatePoint,
    /// 인쇄된 마커의 가로/세로 (정규화 좌표 기준).
    pub size: f32,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BubbleKind {
    /// 학번 입력 영역.
    StudentId,
    /// 일반 문항.
    Question,
}

/// 동일한 의미(예: "1번 문항", "학번 십의자리")를 공유하는 버블 묶음.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BubbleGroup {
    pub id: String,
    pub kind: BubbleKind,
    /// 사람이 보는 라벨 ("1번", "백의자리" 등).
    pub label: String,
    /// 그룹 내 각 선택지의 좌표 (정규화). index 0이 보통 "1" 또는 "0".
    pub bubbles: Vec<TemplatePoint>,
    /// 정답 인덱스 (Question 일 때만 의미). 미설정 시 None.
    pub answer_index: Option<u32>,
    /// 점수 가중치. 기본 1.0.
    #[serde(default = "default_score")]
    pub score: f32,
}

fn default_score() -> f32 {
    1.0
}

/// `templates.json_schema` 컬럼에 그대로 직렬화되는 최상위 구조.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OmrTemplate {
    /// 스키마 버전 — 호환 깨질 때 마이그레이션 트리거.
    #[serde(default = "current_version")]
    pub version: u32,
    pub title: String,
    /// 4개 마커 (TL, TR, BR, BL 순서).
    pub markers: [Marker; 4],
    pub groups: Vec<BubbleGroup>,
}

fn current_version() -> u32 {
    1
}

impl OmrTemplate {
    pub const CURRENT_VERSION: u32 = 1;
}
