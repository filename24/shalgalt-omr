//! Stack several [`BubbleGroup`]s vertically to form a single-digit numeric block (the
//! Шифр column or Section-2 sub-blocks).
//!
//! This module does no coordinate arithmetic of its own — each group's normalized
//! coordinates already carry its row position, so the implementation just dispatches to
//! [`crate::layout::bubble_grid::draw`] for every group. The separate entry point lets
//! P3-01 highlight numeric blocks differently if it needs to.

use printpdf::{FontId, Op};
use shalgalt_core::domain::{template::BubbleGroup, PaperSpec};

use crate::{layout::bubble_grid, style::BubbleStyle};

/// Render every group in `groups` with the same `labels` set, in order.
pub fn draw(
    ops: &mut Vec<Op>,
    groups: &[&BubbleGroup],
    paper: &PaperSpec,
    style: &BubbleStyle,
    labels: &[char],
    font: &FontId,
) {
    for group in groups {
        bubble_grid::draw(ops, group, paper, style, labels, font);
    }
}
