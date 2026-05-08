//! Visual elements for a single OMR card page. Each submodule exposes one or two pure
//! functions that push ops into a caller-owned `&mut Vec<Op>`. All coordinate math goes
//! through [`crate::coords`].
//!
//! The five submodules called out in the P2-06 acceptance criteria:
//! - [`markers`] — four corner alignment markers.
//! - [`header`] — pre-translated title / school / teacher block.
//! - [`bubble_grid`] — a single [`shalgalt_core::domain::BubbleGroup`] painted as a row of
//!   labelled circles.
//! - [`numeric_block`] — a stack of `BubbleGroup`s forming a single-digit numeric block.
//! - [`sidebar`] — the right-edge instruction strip in Mongolian script, rotated 90°.

pub mod bubble_grid;
pub mod header;
pub mod markers;
pub mod numeric_block;
pub mod sidebar;
