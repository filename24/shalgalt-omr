//! OMR computer-vision pipeline.
//!
//! Layering:
//! - `pdf` — pdfium-render rasterization of multi-page PDFs.
//! - `preview` — single-page rasterization used by the P1 template editor.
//! - `threshold` — adaptive Gaussian thresholding + median-blur background removal.
//! - `deskew` — pre-warp `HoughLinesP` rotation correction.
//! - `perspective` — ArUco DICT_6X6_50 detection + `warpPerspective` to canonical.
//! - `bubbles` — per-bubble fill-ratio reading + confidence scoring.
//! - `pipeline` — orchestrates the above and emits [`TaskProgress`] events.
//!
//! `TaskProgress` / `TaskStage` are part of the IPC payload surface and live in
//! `shalgalt_core::domain::progress`. They are re-exported here so the call
//! sites that previously imported them from `shalgalt_cv` keep compiling.
//!
//! Rule 1 / Rule 2 are honored at the crate boundary: every public function takes
//! filesystem paths (never byte buffers crossing IPC) and reports progress via a
//! `tokio::sync::mpsc::Sender<TaskProgress>` rather than blocking the caller.

mod binding;
pub mod bubbles;
pub mod deskew;
pub mod pdf;
pub mod perspective;
mod pipeline;
pub mod preview;
pub mod threshold;

pub use binding::set_pdfium_dir;
pub use shalgalt_core::domain::{TaskProgress, TaskStage};

use std::path::{Path, PathBuf};

use shalgalt_core::domain::{BubbleReading, OmrTemplate, ParsedSheet};
use shalgalt_core::error::AppResult;
use tokio::sync::mpsc;

/// One question group's fill ratios after reading a hand-filled answer sheet (P4-06).
///
/// `fills[i]` is the `[0, 1]` ink ratio of option `i`, in template option order. The
/// helpers translate those raw fills into the answer-key decision the UI needs.
#[derive(Debug, Clone)]
pub struct GroupReading {
    /// Matches [`shalgalt_core::domain::BubbleGroup::id`].
    pub group_id: String,
    /// Per-option fill ratio in `[0, 1]`, in template option order.
    pub fills: Vec<f32>,
}

impl GroupReading {
    /// Index of the most-filled option, or `None` when the group reads as blank
    /// (every option below [`BubbleReading::FILL_UNFILLED_MAX`]). A blank group maps
    /// to `-1` at the IPC boundary.
    pub fn most_filled_index(&self) -> Option<usize> {
        let (idx, &best) = self
            .fills
            .iter()
            .enumerate()
            .max_by(|(_, a), (_, b)| a.total_cmp(b))?;
        if best < BubbleReading::FILL_UNFILLED_MAX {
            None
        } else {
            Some(idx)
        }
    }

    /// True when any option sits in the `[0.35, 0.65]` uncertain band — the same band
    /// the grading engine uses for `needs_review`. The UI flags these for manual
    /// confirmation before the answer key is saved.
    pub fn is_uncertain(&self) -> bool {
        let band = BubbleReading::FILL_UNFILLED_MAX..=BubbleReading::FILL_FILLED_MIN;
        self.fills.iter().any(|f| band.contains(f))
    }
}

/// Result of reading a single hand-filled answer sheet (P4-06). One [`GroupReading`]
/// per question group, in template order, plus the page count of the source PDF.
#[derive(Debug, Clone)]
pub struct AnswerKeyReading {
    pub groups: Vec<GroupReading>,
    pub page_count: u32,
}

/// End-to-end OMR processing facade — flatten an ordered batch of sources (each a
/// multi-page PDF or a single scanned image) into one continuous page sequence, align
/// each page against the four template ArUco markers, read the bubbles, and stream
/// progress events through `progress_tx` while the work happens.
///
/// Passing several images grades them as a multi-page batch in upload order; a single
/// PDF behaves exactly as before. `cache_dir` is where intermediate page rasters land —
/// the desktop app passes its `app_cache_dir`; tests typically pass a `tempfile::TempDir`.
pub async fn process_sources(
    source_paths: &[PathBuf],
    template: &OmrTemplate,
    progress_tx: mpsc::Sender<TaskProgress>,
    task_id: &str,
    cache_dir: &Path,
) -> AppResult<Vec<ParsedSheet>> {
    pipeline::run(
        source_paths.to_vec(),
        template.clone(),
        progress_tx,
        task_id.to_string(),
        cache_dir.to_path_buf(),
    )
    .await
}

/// Read one hand-filled OMR sheet as an answer key (P4-06).
///
/// The teacher prints the card, fills the correct bubbles by hand, and scans it; this
/// reuses the grading pipeline's CV stages (ArUco alignment + adaptive threshold +
/// per-bubble fill) to recover the canonical answers. The source PDF MUST be a single
/// page; multi-page input returns [`AppError::AnswerKeyMultiPage`](shalgalt_core::error::AppError::AnswerKeyMultiPage).
/// Progress is streamed through `progress_tx` on the same channel grading uses.
///
/// `cache_dir` receives the page raster (reused by the desktop command as the review
/// preview); the desktop app passes its `app_cache_dir`.
pub async fn read_answer_key(
    pdf_path: &Path,
    template: &OmrTemplate,
    progress_tx: mpsc::Sender<TaskProgress>,
    task_id: &str,
    cache_dir: &Path,
) -> AppResult<AnswerKeyReading> {
    pipeline::run_answer_key(
        pdf_path.to_path_buf(),
        template.clone(),
        progress_tx,
        task_id.to_string(),
        cache_dir.to_path_buf(),
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::*;

    fn group(fills: &[f32]) -> GroupReading {
        GroupReading {
            group_id: "q1".into(),
            fills: fills.to_vec(),
        }
    }

    #[test]
    fn most_filled_index_picks_the_marked_option() {
        let g = group(&[0.02, 0.05, 0.92, 0.01]);
        assert_eq!(g.most_filled_index(), Some(2));
    }

    #[test]
    fn blank_group_maps_to_none() {
        // Every option below the unfilled threshold ⇒ no answer.
        let g = group(&[0.10, 0.08, 0.12, 0.05]);
        assert_eq!(g.most_filled_index(), None);
    }

    #[test]
    fn uncertain_band_member_is_flagged_but_still_indexed() {
        // 0.50 sits inside [0.35, 0.65]: uncertain, yet it is still the best option.
        let g = group(&[0.03, 0.50, 0.04]);
        assert!(g.is_uncertain());
        assert_eq!(g.most_filled_index(), Some(1));
    }

    #[test]
    fn confident_marks_are_not_uncertain() {
        let g = group(&[0.02, 0.90, 0.03]);
        assert!(!g.is_uncertain());
    }

    #[test]
    fn band_edges_are_inclusive() {
        assert!(group(&[0.35, 0.0]).is_uncertain());
        assert!(group(&[0.65, 0.0]).is_uncertain());
    }
}
