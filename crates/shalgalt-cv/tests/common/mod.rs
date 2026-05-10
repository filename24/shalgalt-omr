//! Shared helpers for `shalgalt-cv` integration tests.
//!
//! `synth` generates synthetic scanned-OMR raster images so the pipeline tests do
//! not depend on real scan corpora (which we do not yet have locally — see
//! `fixtures/README.md`).

pub mod synth;
