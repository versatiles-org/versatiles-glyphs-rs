//! This module provides Protobuf-based data structures and utilities for glyph handling.
//!
//! It contains structures for individual glyph data, font stacks, and methods for serializing
//! or deserializing these structures as Protobuf-encoded data.

mod fontstack;
mod glyph;
mod glyphs;
mod pbf_glyphs;

/// Re-export of the [`Glyph`](glyph::Glyph) struct as `PbfGlyph`.
pub use glyph::Glyph as PbfGlyph;

/// A helper structure for building and encoding a `Fontstack`
/// into a Protobuf buffer.
pub use pbf_glyphs::PbfGlyphs;
