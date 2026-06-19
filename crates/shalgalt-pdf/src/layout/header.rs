//! Top-of-page header band.
//!
//! Two-column layout, modelled after real Mongolian school OMR cards:
//! - **Left**: exam title (centered above the cipher block — see below)
//! - **Right**: school + teacher lines (right-aligned)
//!
//! The title sits above the cipher block (Шифр) rather than at the page's left margin.
//! The original left-aligned title bled into the top-left ArUco marker's quiet zone,
//! which lowers detection rates because the legacy `cv::aruco::detect_markers` and
//! `cv::objdetect::ArucoDetector` algorithms expect a uniform white border one cell
//! wide around each marker. Centring the title on the cipher block's normalized
//! x-midpoint moves it out of all four corner-marker dead zones.
//!
//! Splitting the header into left + right blocks keeps the band height capped at the
//! taller of the two columns — about 10 mm — so the cipher area (`startY = 0.08`,
//! ≈ 24 mm from the page top) stays clear of header text.
//!
//! Every string arrives pre-translated via [`HeaderText`]. The renderer never imports an
//! i18n table.

use printpdf::FontId;
use shalgalt_core::domain::{BubbleGroup, BubbleKind, PaperSpec};

use crate::{canvas::Canvas, style::HeaderText};

/// Title font size — modest reduction from 18 pt so the header band stays compact and
/// leaves room for additional metadata lines without crowding the cipher block below.
const TITLE_PT: f64 = 14.0;

/// Subtitle font size (left column, second line).
const SUBTITLE_PT: f64 = 9.0;

/// Right-column font size (school + teacher).
const META_PT: f64 = 9.0;

/// Vertical offset from the page top to the title baseline.
const TITLE_BASELINE_FROM_TOP_MM: f64 = 6.0;

/// Vertical step from the title baseline to the subtitle baseline.
const SUBTITLE_GAP_MM: f64 = 4.5;

/// Vertical step between consecutive right-column lines (school → teacher).
const META_LINE_GAP_MM: f64 = 4.0;

/// Draw the header band. An empty [`HeaderText`] adds no ops.
///
/// `groups` is consulted to compute the cipher-block x-midpoint that the title is
/// centred on. When the template has no `BubbleKind::StudentId` groups (custom
/// templates without a cipher block), the title falls back to page-centre — that
/// still keeps it clear of all four corner markers.
pub fn draw(
    canvas: &mut Canvas,
    header: &HeaderText,
    groups: &[BubbleGroup],
    paper: &PaperSpec,
    font: &FontId,
) {
    if header == &HeaderText::default() {
        return;
    }

    let title_baseline_y = paper.height_mm - paper.margin_mm - TITLE_BASELINE_FROM_TOP_MM;
    let title_center_x = title_anchor_x_mm(groups, paper);
    let right_x = paper.width_mm - paper.margin_mm;

    // Left column — title (centred over the cipher block) + subtitle directly under it.
    if let Some(title) = header.title.as_deref() {
        canvas.text_centered_x(title_center_x, title_baseline_y, title, font, TITLE_PT);
    }
    if let Some(subtitle) = header.subtitle.as_deref() {
        canvas.text_centered_x(
            title_center_x,
            title_baseline_y - SUBTITLE_GAP_MM,
            subtitle,
            font,
            SUBTITLE_PT,
        );
    }

    // Right column — school + teacher, right-aligned. School sits on the title's
    // baseline so the whole band stays compact; teacher drops one line below.
    if let Some(school) = header.school.as_deref() {
        canvas.text_right_aligned(right_x, title_baseline_y, school, font, META_PT);
    }
    if let Some(teacher) = header.teacher.as_deref() {
        canvas.text_right_aligned(
            right_x,
            title_baseline_y - META_LINE_GAP_MM,
            teacher,
            font,
            META_PT,
        );
    }
}

/// Compute the x-coordinate (mm, page-absolute) that the title should be centred on.
/// Walks every `StudentId` group's bubble positions and returns the midpoint of their
/// combined x-extent. Falls back to page-centre when no cipher block is present.
fn title_anchor_x_mm(groups: &[BubbleGroup], paper: &PaperSpec) -> f64 {
    let cipher_xs = groups
        .iter()
        .filter(|g| matches!(g.kind, BubbleKind::StudentId))
        .flat_map(|g| g.bubbles.iter().map(|b| b.x as f64));

    let mut min_x = f64::INFINITY;
    let mut max_x = f64::NEG_INFINITY;
    for x in cipher_xs {
        if x < min_x {
            min_x = x;
        }
        if x > max_x {
            max_x = x;
        }
    }

    if min_x.is_finite() && max_x.is_finite() {
        let mid_norm = (min_x + max_x) / 2.0;
        return mid_norm * paper.width_mm;
    }

    paper.width_mm / 2.0
}
