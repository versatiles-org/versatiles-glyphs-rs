//! Protocol Buffers and their generated code for glyph data.
//!
//! It contains structures for individual glyph data, font stacks, and methods for serializing
//! or deserializing these structures as Protobuf-encoded data.

mod fontstack;
mod glyph;
mod glyphs;

pub use glyph::PbfGlyph;
pub use glyphs::PbfGlyphs;
