use super::file_entry::FontFileEntry;
use crate::{glyph::render_glyph, protobuf::PbfGlyphs};
use anyhow::Result;
use std::collections::HashMap;

pub const GLYPH_BLOCK_SIZE: u32 = 256;

/// A block of up to 256 characters that can be rendered to a PBF file.
pub struct GlyphBlock<'a> {
	start_index: u32,
	glyphs: HashMap<u8, &'a FontFileEntry<'a>>,
}

impl<'a> GlyphBlock<'a> {
	pub fn new(start_index: u32) -> GlyphBlock<'a> {
		GlyphBlock {
			start_index,
			glyphs: HashMap::new(),
		}
	}

	pub fn set_glyph_font(&mut self, char_index: u8, font: &'a FontFileEntry<'a>) {
		self.glyphs.entry(char_index).or_insert(font);
	}

	pub fn len(&self) -> usize {
		self.glyphs.len()
	}

	#[allow(dead_code)]
	pub fn is_empty(&self) -> bool {
		self.glyphs.is_empty()
	}

	fn range(&self) -> String {
		format!(
			"{}-{}",
			self.start_index,
			self.start_index + GLYPH_BLOCK_SIZE - 1
		)
	}

	pub fn render(&self, font_name: String) -> Result<Vec<u8>> {
		let mut glyphs = PbfGlyphs::new(font_name, self.range());

		for (index, font) in self.glyphs.iter() {
			let codepoint = self.start_index + *index as u32;
			if let Some(glyph) = render_glyph(&font.face, codepoint) {
				glyphs.push(glyph);
			}
		}

		glyphs.into_vec()
	}

	pub fn filename(&self) -> String {
		format!("{}.pbf", self.range())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	const VALID_FONT: &[u8] = include_bytes!("../../testdata/Fira Sans - Regular.ttf");

	// Helper to create a FontFileEntry from the test font bytes.
	fn create_font_file_entry<'a>() -> FontFileEntry<'a> {
		FontFileEntry::new(VALID_FONT.to_vec()).expect("Valid font should parse")
	}

	#[test]
	fn test_new_and_set_char_font() {
		// Create a new GlyphBlock with start_index 0
		let mut block = GlyphBlock::new(0);
		assert!(block.is_empty());
		assert_eq!(block.len(), 0);

		// Create a FontFileEntry from valid font data
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
		// Create a GlyphBlock and add a FontFileEntry for a known character.
		let mut block = GlyphBlock::new(0);
		let font_entry = create_font_file_entry();
		block.set_glyph_font(65, &font_entry);

		// Render the block with a given font name.
		let render_result = block.render("TestFont".to_string());
		assert!(render_result.is_ok());
		let out_data = render_result.unwrap();
		assert!(!out_data.is_empty());
	}
}
