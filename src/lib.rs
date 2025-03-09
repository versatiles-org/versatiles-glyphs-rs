//! VersaTiles Glyphs Library
//!
//! This crate provides functionality for parsing fonts, rendering glyphs as SDFs,
//! and writing them to disk or a tar archive. It also supports indexing and
//! generating metadata for fonts, making it easier to work with multiple font files
//! in a single pipeline.

pub mod font;
pub mod geometry;
pub mod protobuf;
pub mod render;
pub mod utils;
pub mod writer;
