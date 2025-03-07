//! Font-handling module for VersaTiles Glyphs.
//!
//! This module offers:
//!
//! - Low-level constructs for reading and parsing individual font files ([`FontFileEntry`](font::FontFileEntry)).  
//! - Structures to represent glyph blocks ([`GlyphBlock`](font::GlyphBlock)) and build indices ([`build_index_json`](font::build_index_json), [`build_font_families_json`](font::build_font_families_json)).  
//! - A high-level [`FontManager`](font::FontManager) for orchestrating multiple fonts, rendering, and metadata.  
//! - Metadata extraction utilities ([`FontMetadata`](font::FontMetadata) and [`parse_font_name`](font::parse_font_name)) to identify the font’s
//!   family, style, weight, width, and codepoints.  
//! - A [`FontWrapper`](font::FontWrapper) to combine multiple files into one logical font (e.g., different languages).  

mod file_entry;
mod glyph_block;
mod index_files;
mod manager;
mod metadata;
mod parse_font_name;
mod wrapper;

pub use file_entry::FontFileEntry;
pub use glyph_block::{GlyphBlock, GLYPH_BLOCK_SIZE};
pub use index_files::{build_font_families_json, build_index_json};
pub use manager::FontManager;
pub use metadata::FontMetadata;
pub use parse_font_name::parse_font_name;
pub use wrapper::FontWrapper;
