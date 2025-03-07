//! VersaTiles Glyphs Library
//!
//! This crate provides functionality for parsing fonts, rendering glyphs as SDFs,
//! and writing them to disk or a tar archive. It also supports indexing and
//! generating metadata for fonts, making it easier to work with multiple font files
//! in a single pipeline.

/// Font parsing, management, and metadata.
pub mod font;

/// Geometric primitives and operations used for glyph outlines.
pub mod geometry;

/// High-level glyph representation and outline building.
pub mod glyph;

/// Protocol Buffers and their generated code for glyph data.
pub mod protobuf;

/// Rendering traits and implementations for generating SDF.
pub mod renderer;

/// Utility functions and structures for file paths, progress bars, etc.
pub mod utils;

/// Writers for storing glyph data in files or tar archives.
pub mod writer;
