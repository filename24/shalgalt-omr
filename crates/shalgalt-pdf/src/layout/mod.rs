//! Visual elements for a single OMR card page. Each submodule exposes one or two pure
//! functions that push ops onto a caller-owned [`crate::canvas::Canvas`]. All coordinate
//! math goes through [`crate::coords`].
//!
//! Submodules:
//! - [`markers`] — four corner alignment markers.
//! - [`header`] — pre-translated title / school / teacher block.
//! - [`bubble_grid`] — a single [`shalgalt_core::domain::BubbleGroup`] painted as a row of
//!   empty circles. Labels are NOT emitted here — see [`labels`].
//! - [`labels`] — per-section column headers (`A B C D` / `0–9`) and per-group row labels
//!   drawn at the [`crate::render_template`] level.
//! - [`numeric_block`] — a stack of `BubbleGroup`s forming a single-digit numeric block.
//! - [`sidebar`] — the right-edge instruction strip in Mongolian script, rotated 90°.

pub mod bubble_grid;
pub mod header;
pub mod labels;
pub mod manual_entry;
pub mod markers;
pub mod numeric_block;
pub mod section_headers;
pub mod sidebar;
