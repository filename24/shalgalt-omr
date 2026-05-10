//! Layout map sidecar — AMC `.xy`-style record of every printed bubble + marker
//! position in PDF millimetres.
//!
//! AMC's pdfTeX wrapper writes a `.xy` file alongside the PDF mapping each bubble
//! identifier to its page coordinates. The grader reads from that file rather than
//! re-projecting from the source template, so a renderer rounding tweak (or anything
//! that touches `coords::project`) cannot silently desynchronise the grader from the
//! sheet that actually came out of the printer.
//!
//! `shalgalt-pdf` adopts the same pattern. [`render_template_with_map`] returns the PDF
//! bytes alongside a [`LayoutMap`]; [`LayoutMap::from_template`] derives the map without
//! rendering. Both paths share a single helper, [`bubble_label_for`], so the label
//! captured in the map is exactly the character drawn inside the circle.
//!
//! See [ADR 0007](../../../docs/adr/0007-canvas-wrapper-and-layout-modules.md) for the
//! rendering pipeline and [`docs/MONGOLIAN_OMR_SPEC.md`](../../../docs/MONGOLIAN_OMR_SPEC.md)
//! for the spec the coordinates implement.

use serde::{Deserialize, Serialize};
use shalgalt_core::domain::{
    paper::PaperSpec,
    template::{BubbleGroup, BubbleKind, MarkerKind, OmrTemplate},
};

use crate::{coords, style::BubbleStyle, PdfOptions};

/// Position of a printed corner marker in PDF millimetres (origin top-left of the page
/// in this coordinate system — y increases downward, matching the convention of every
/// camera + scanner pipeline that consumes the output).
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MarkerEntry {
    pub id: String,
    pub kind: MarkerKind,
    /// Centre x in PDF mm.
    pub x_mm: f64,
    /// Centre y in PDF mm.
    pub y_mm: f64,
    /// Half-extent (square half-side) in mm.
    pub half_mm: f64,
}

/// Position of a single printed bubble in PDF millimetres.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BubbleEntry {
    /// Composite identifier: `"{group_id}:{index_in_group}"`. Stable across renderings
    /// of the same template — the grader uses this to map a CV detection back to a
    /// `BubbleGroup` answer index.
    pub id: String,
    pub group_id: String,
    pub index_in_group: usize,
    pub kind: BubbleKind,
    /// Character printed inside the circle (`'A'..='E'` for question rows, `'0'..='9'`
    /// for cipher / numeric rows). Empty groups (no labels supplied) produce `' '`.
    pub label: char,
    /// Centre x in PDF mm.
    pub x_mm: f64,
    /// Centre y in PDF mm.
    pub y_mm: f64,
    pub r_mm: f64,
}

/// Page-side coordinate map written by the renderer alongside the PDF.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LayoutMap {
    pub paper: PaperSpec,
    pub markers: Vec<MarkerEntry>,
    pub bubbles: Vec<BubbleEntry>,
}

impl LayoutMap {
    /// Compute the layout map deterministically from the template + options. Pure
    /// function — no PDF generation — so the grader can call it ahead of time when it
    /// needs coordinates without paying for a render pass.
    pub fn from_template(template: &OmrTemplate, opts: &PdfOptions) -> Self {
        let paper = opts.paper;
        let style = opts.bubble_style;
        let r_mm = style.diameter_mm * 0.5;

        let markers = template
            .markers
            .iter()
            .map(|marker| {
                let (cx, cy) =
                    coords::to_page_mm(marker.position.x as f64, marker.position.y as f64, &paper);
                let half_mm = marker.size as f64 * paper.width_mm.min(paper.height_mm) * 0.5;
                MarkerEntry {
                    id: marker.id.clone(),
                    kind: marker.kind,
                    x_mm: cx,
                    y_mm: cy,
                    half_mm,
                }
            })
            .collect();

        let mut bubbles: Vec<BubbleEntry> = Vec::new();
        for group in &template.groups {
            let labels = bubble_label_set(group, opts);
            for (i, point) in group.bubbles.iter().enumerate() {
                let (cx, cy) = coords::project(point.x as f64, point.y as f64, &paper);
                bubbles.push(BubbleEntry {
                    id: format!("{}:{}", group.id, i),
                    group_id: group.id.clone(),
                    index_in_group: i,
                    kind: group.kind,
                    label: labels.get(i).copied().unwrap_or(' '),
                    x_mm: cx.0 as f64,
                    y_mm: cy.0 as f64,
                    r_mm,
                });
            }
        }

        LayoutMap {
            paper,
            markers,
            bubbles,
        }
    }

    /// Look up a bubble by `(group_id, index)`. `None` when the pair does not exist —
    /// callers that loaded a stale map alongside a refreshed template should treat this
    /// as a hard error.
    pub fn find_bubble(&self, group_id: &str, index_in_group: usize) -> Option<&BubbleEntry> {
        self.bubbles
            .iter()
            .find(|b| b.group_id == group_id && b.index_in_group == index_in_group)
    }

    /// Total bubble count. Provided for symmetry with `template.groups.iter().map(...)
    /// .sum()` so grading code does not duplicate the iterator chain.
    pub fn bubble_count(&self) -> usize {
        self.bubbles.len()
    }
}

/// Pick the character printed inside the *i*-th bubble of `group`. Mirrors the
/// label-selection rule in `lib.rs::build_page_ops` so the map matches the printed
/// glyph exactly.
fn bubble_label_set<'a>(group: &BubbleGroup, opts: &'a PdfOptions) -> &'a [char] {
    match group.kind {
        BubbleKind::StudentId => &opts.digit_labels,
        BubbleKind::Question => {
            if group.bubbles.len() > opts.choice_labels.len() {
                &opts.digit_labels
            } else {
                &opts.choice_labels
            }
        }
    }
}

/// Look up the printed label for a single bubble at index `i` inside `group`. Wraps
/// [`bubble_label_set`] for callers that just need one character.
pub fn bubble_label_for(group: &BubbleGroup, opts: &PdfOptions, i: usize) -> char {
    bubble_label_set(group, opts).get(i).copied().unwrap_or(' ')
}

/// Convenience accessor — the bubble radius implied by `BubbleStyle`. Centralised here
/// so the grader and the renderer agree to the byte on what "the radius" means.
pub fn bubble_radius_mm(style: &BubbleStyle) -> f64 {
    style.diameter_mm * 0.5
}

#[cfg(test)]
mod tests {
    use super::*;
    use shalgalt_core::domain::template::{Marker, TemplatePoint};

    fn fixture_template() -> OmrTemplate {
        OmrTemplate {
            version: OmrTemplate::CURRENT_VERSION,
            title: "test".into(),
            markers: [
                marker("m-tl", 0.04, 0.04),
                marker("m-tr", 0.96, 0.04),
                marker("m-br", 0.96, 0.96),
                marker("m-bl", 0.04, 0.96),
            ],
            groups: vec![BubbleGroup {
                id: "q-1".into(),
                kind: BubbleKind::Question,
                label: "1".into(),
                bubbles: (0..5)
                    .map(|i| TemplatePoint {
                        x: 0.07 + i as f32 * 0.032,
                        y: 0.27,
                    })
                    .collect(),
                answer_index: None,
                score: 1.0,
                section: None,
            }],
        }
    }

    fn marker(id: &str, x: f32, y: f32) -> Marker {
        Marker {
            id: id.into(),
            position: TemplatePoint { x, y },
            size: 0.02,
            kind: MarkerKind::Square,
        }
    }

    #[test]
    fn from_template_maps_every_marker() {
        let opts = PdfOptions::default();
        let map = LayoutMap::from_template(&fixture_template(), &opts);
        assert_eq!(map.markers.len(), 4);
        let ids: Vec<&str> = map.markers.iter().map(|m| m.id.as_str()).collect();
        assert_eq!(ids, ["m-tl", "m-tr", "m-br", "m-bl"]);
    }

    #[test]
    fn from_template_assigns_choice_labels_a_through_e() {
        let opts = PdfOptions::default();
        let map = LayoutMap::from_template(&fixture_template(), &opts);
        let labels: Vec<char> = map.bubbles.iter().map(|b| b.label).collect();
        assert_eq!(labels, ['A', 'B', 'C', 'D', 'E']);
    }

    #[test]
    fn from_template_bubbles_match_coords_project() {
        let opts = PdfOptions::default();
        let template = fixture_template();
        let map = LayoutMap::from_template(&template, &opts);
        let first = &template.groups[0].bubbles[0];
        let (expected_x, expected_y) = coords::project(first.x as f64, first.y as f64, &opts.paper);
        let entry = &map.bubbles[0];
        assert!((entry.x_mm - expected_x.0 as f64).abs() < 1e-6);
        assert!((entry.y_mm - expected_y.0 as f64).abs() < 1e-6);
        assert_eq!(entry.r_mm, opts.bubble_style.diameter_mm * 0.5);
    }

    #[test]
    fn find_bubble_round_trips_by_id() {
        let opts = PdfOptions::default();
        let map = LayoutMap::from_template(&fixture_template(), &opts);
        let entry = map.find_bubble("q-1", 2).unwrap();
        assert_eq!(entry.label, 'C');
        assert_eq!(entry.index_in_group, 2);
        assert!(map.find_bubble("q-1", 99).is_none());
        assert!(map.find_bubble("nonexistent", 0).is_none());
    }

    #[test]
    fn ten_bubble_question_group_uses_digit_labels() {
        // A Question group with 10 bubbles exceeds choice_labels.len() (5),
        // so it falls back to digit labels — same heuristic as the renderer.
        let opts = PdfOptions::default();
        let mut t = fixture_template();
        t.groups[0].bubbles = (0..10)
            .map(|i| TemplatePoint {
                x: 0.07 + i as f32 * 0.032,
                y: 0.27,
            })
            .collect();
        let map = LayoutMap::from_template(&t, &opts);
        let labels: Vec<char> = map.bubbles.iter().map(|b| b.label).collect();
        assert_eq!(labels, ['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);
    }

    #[test]
    fn map_serializes_round_trip_via_serde_json() {
        // JSON's f64 string encoding loses a sub-ulp of precision, so the round-trip
        // is "approximately equal" rather than bit-identical. Tolerance 1e-6 mm is
        // four orders of magnitude tighter than CV detection accuracy.
        let opts = PdfOptions::default();
        let original = LayoutMap::from_template(&fixture_template(), &opts);
        let json = serde_json::to_string(&original).expect("serialize");
        let parsed: LayoutMap = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(parsed.bubbles.len(), original.bubbles.len());
        assert_eq!(parsed.markers.len(), original.markers.len());
        for (p, o) in parsed.markers.iter().zip(original.markers.iter()) {
            assert_eq!(p.id, o.id);
            assert_eq!(p.kind, o.kind);
            assert!((p.x_mm - o.x_mm).abs() < 1e-6);
            assert!((p.y_mm - o.y_mm).abs() < 1e-6);
            assert!((p.half_mm - o.half_mm).abs() < 1e-6);
        }
        for (p, o) in parsed.bubbles.iter().zip(original.bubbles.iter()) {
            assert_eq!(p.id, o.id);
            assert_eq!(p.group_id, o.group_id);
            assert_eq!(p.index_in_group, o.index_in_group);
            assert_eq!(p.kind, o.kind);
            assert_eq!(p.label, o.label);
            assert!((p.x_mm - o.x_mm).abs() < 1e-6);
            assert!((p.y_mm - o.y_mm).abs() < 1e-6);
            assert!((p.r_mm - o.r_mm).abs() < 1e-6);
        }
    }
}
