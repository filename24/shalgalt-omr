//! Top-of-page header band.
//!
//! Two-column layout, modelled after real Mongolian school OMR cards:
//! - **Left**: exam title (18 pt) + optional subtitle/grade tag (12 pt)
//! - **Right**: school + teacher lines (11 pt, right-aligned)
//!
//! Splitting the header into left + right blocks keeps the band height capped at the
//! taller of the two columns — about 10 mm — so the cipher area (`startY = 0.08`,
//! ≈ 24 mm from the page top) stays clear of header text.
//!
//! Every string arrives pre-translated via [`HeaderText`]. The renderer never imports an
//! i18n table.

use printpdf::FontId;
use shalgalt_core::domain::PaperSpec;

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
pub fn draw(canvas: &mut Canvas, header: &HeaderText, paper: &PaperSpec, font: &FontId) {
    if header == &HeaderText::default() {
        return;
    }

    let title_baseline_y = paper.height_mm - paper.margin_mm - TITLE_BASELINE_FROM_TOP_MM;
    let left_x = paper.margin_mm;
    let right_x = paper.width_mm - paper.margin_mm;

    // Left column — title (18 pt) + subtitle (12 pt).
    if let Some(title) = header.title.as_deref() {
        canvas.text(left_x, title_baseline_y, title, font, TITLE_PT);
    }
    if let Some(subtitle) = header.subtitle.as_deref() {
        canvas.text(
            left_x,
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
