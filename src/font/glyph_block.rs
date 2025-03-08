use super::file_entry::FontFileEntry;
use crate::{
	protobuf::PbfGlyphs,
	render::{render_glyph, RendererTrait},
};
use anyhow::Result;
use std::collections::HashMap;

/// The number of glyphs in each block, corresponding to a range of 256 codepoints.
pub const GLYPH_BLOCK_SIZE: u32 = 256;

/// Represents a block of glyphs (up to 256) that can be rendered into a `.pbf` file.
/// Each block tracks which font file is responsible for each character.
pub struct GlyphBlock<'a> {
	/// The start of the codepoint range for this block.
	pub start_index: u32,
	/// A map from the codepoint offset (`0..=255`) to the [`FontFileEntry`] that provides the glyph.
	pub glyphs: HashMap<u8, &'a FontFileEntry<'a>>,
}

impl<'a> GlyphBlock<'a> {
	/// Creates a new, empty [`GlyphBlock`] with the specified start index.
	///
	/// The block will cover the codepoints from `start_index` to
	/// `start_index + GLYPH_BLOCK_SIZE - 1`.
	pub fn new(start_index: u32) -> Self {
		GlyphBlock {
			start_index,
			glyphs: HashMap::new(),
		}
	}

	/// Associates a single character index (0–255) with a particular font file.
	///
	/// This indicates that the specified codepoint (based on `start_index + char_index`)
	/// will be rendered using the provided font face data.
	pub fn set_glyph_font(&mut self, char_index: u8, font: &'a FontFileEntry<'a>) {
		self.glyphs.entry(char_index).or_insert(font);
	}

	/// Returns the number of codepoints within this block that are mapped to a font.
	pub fn len(&self) -> usize {
		self.glyphs.len()
	}

	/// Returns `true` if no characters in this block are mapped to a font.
	#[allow(dead_code)]
	pub fn is_empty(&self) -> bool {
		self.glyphs.is_empty()
	}

	/// Provides a string representation of this block's codepoint range.
	fn range(&self) -> String {
		format!(
			"{}-{}",
			self.start_index,
			self.start_index + GLYPH_BLOCK_SIZE - 1
		)
	}

	/// Renders all glyphs in this block using the provided [`RendererTrait`] implementation.
	///
	/// A [`PbfGlyphs`] structure is created to store the glyph data, which is then serialized
	/// into a `Vec<u8>`.
	///
	/// # Errors
	///
	/// Returns an error if glyph rendering fails.
	pub fn render(&self, font_name: String, renderer: &impl RendererTrait) -> Result<Vec<u8>> {
		let mut glyphs = PbfGlyphs::new(font_name, self.range());

		for (char_index, font_entry) in &self.glyphs {
			let codepoint = self.start_index + (*char_index as u32);
			if let Some(glyph) = render_glyph(&font_entry.face, codepoint, renderer) {
				glyphs.push(glyph);
			}
		}

		glyphs.into_vec()
	}

	/// Generates a filename for the `.pbf` file representing this block.
	///
	/// For instance, if the block covers `0–255`, the filename would be `0-255.pbf`.
	pub fn filename(&self) -> String {
		format!("{}.pbf", self.range())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::render::RendererDummy;

	const VALID_FONT: &[u8] = include_bytes!("../../testdata/Fira Sans - Regular.ttf");

	// Helper to create a FontFileEntry from the test font bytes.
	fn create_font_file_entry<'a>() -> FontFileEntry<'a> {
		FontFileEntry::new(VALID_FONT.to_vec()).expect("Valid font should parse")
	}

	#[test]
	fn test_new_and_set_char_font() {
		// Create a new GlyphBlock with start_index 0.
		let mut block = GlyphBlock::new(0);
		assert!(block.is_empty());
		assert_eq!(block.len(), 0);

		// Create a FontFileEntry from valid font data.
		let font_entry = create_font_file_entry();
		block.set_glyph_font(65, &font_entry);
		assert!(!block.is_empty());
		assert_eq!(block.len(), 1);
	}

	#[test]
	fn test_range_and_filename() {
		let start_index = 256;
		let block = GlyphBlock::new(start_index);

		let expected_range = format!("{}-{}", start_index, start_index + GLYPH_BLOCK_SIZE - 1);
		assert_eq!(block.range(), expected_range);
		assert_eq!(block.filename(), format!("{}.pbf", expected_range));
	}

	#[test]
	fn test_render_returns_data() {
		let mut block = GlyphBlock::new(0);
		let font_entry = create_font_file_entry();
		block.set_glyph_font(65, &font_entry);

		let render_result = block.render("TestFont".to_string(), &RendererDummy {});
		assert!(render_result.is_ok());
		let out_data = render_result.unwrap();
		assert!(!out_data.is_empty());
	}
}
