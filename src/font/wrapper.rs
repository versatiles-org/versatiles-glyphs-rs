//! Manages a collection of one or more [`FontFileEntry`] instances that represent a logical font.
//!
//! This wrapper abstracts multiple font files that share the same "family" identity
//! (for example, different languages). It provides methods to load font
//! data from file paths, retrieve metadata, and generate glyph blocks for rendering.

use super::{FontFileEntry, FontMetadata, GlyphBlock, GLYPH_BLOCK_SIZE};
use anyhow::{Context, Result};
use std::{collections::HashMap, path::PathBuf};

/// A wrapper around one or more [`FontFileEntry`] instances.  
/// Each [`FontWrapper`] is effectively a "logical" font that can span
/// multiple font files (e.g., for different languages).
#[derive(Default)]
pub struct FontWrapper<'a> {
	/// Collection of all font files that share the same logical font identity.
	pub files: Vec<FontFileEntry<'a>>,
}

impl<'a> FontWrapper<'a> {
	/// Adds a single [`FontFileEntry`] to this wrapper.
	pub fn add_file(&mut self, file: FontFileEntry<'a>) {
		self.files.push(file);
	}

	/// Adds multiple font files by reading them from the given file paths.
	///
	/// # Errors
	///
	/// Returns an error if reading or parsing any of the font files fails.
	pub fn add_paths(&mut self, sources: &[PathBuf]) -> Result<()> {
		for path in sources {
			let data =
				std::fs::read(path).with_context(|| format!("reading font file \"{path:?}\""))?;
			self.files.push(FontFileEntry::new(data)?);
		}
		Ok(())
	}

	/// Gathers all codepoints from every contained [`FontFileEntry`], grouping them
	/// into [`GlyphBlock`]s of size [`GLYPH_BLOCK_SIZE`].
	///
	/// This is essential for rendering, as each block corresponds to a `.pbf` file
	/// covering a particular range of Unicode codepoints.
	pub fn get_blocks(&'a self) -> Vec<GlyphBlock<'a>> {
		let mut blocks = HashMap::<u32, GlyphBlock<'a>>::new();
		for i in 0..256 {
			blocks.insert(i, GlyphBlock::new(i * GLYPH_BLOCK_SIZE));
		}

		// For each file, for each codepoint, place the codepoint into its corresponding block.
		for font_file in &self.files {
			for &codepoint in &font_file.metadata.codepoints {
				let block_index = codepoint / GLYPH_BLOCK_SIZE;
				let char_index = (codepoint % GLYPH_BLOCK_SIZE) as u8;
				let block = blocks
					.entry(block_index)
					.or_insert_with(|| GlyphBlock::new(block_index * GLYPH_BLOCK_SIZE));
				block.set_glyph_font(char_index, font_file);
			}
		}

		blocks.into_values().collect()
	}

	/// Returns the [`FontMetadata`] of the first font file in this wrapper.
	///
	/// Assumes at least one file is present; if this wrapper is empty, calling
	/// this function would panic.
	pub fn get_metadata(&self) -> &FontMetadata {
		&self.files.first().unwrap().metadata
	}
}

impl<'a> From<FontFileEntry<'a>> for FontWrapper<'a> {
	/// Creates a new [`FontWrapper`] from a single [`FontFileEntry`].
	fn from(file: FontFileEntry<'a>) -> Self {
		let mut font = FontWrapper::default();
		font.add_file(file);
		font
	}
}

impl TryFrom<&[PathBuf]> for FontWrapper<'_> {
	type Error = anyhow::Error;

	/// Attempts to create a new [`FontWrapper`] from a slice of file paths.
	///
	/// # Errors
	///
	/// Returns an error if reading or parsing any of the font files fails.
	fn try_from(paths: &[PathBuf]) -> Result<Self> {
		let mut font = FontWrapper::default();
		font.add_paths(paths)?;
		Ok(font)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// Helper function to create a FontFileEntry from a known valid test font.
	fn create_test_font_file_entry<'a>() -> FontFileEntry<'a> {
		FontFileEntry::new(include_bytes!("../../testdata/Fira Sans - Regular.ttf").to_vec()).unwrap()
	}

	#[test]
	fn test_add_file_and_get_metadata() {
		let wrapper = FontWrapper::from(create_test_font_file_entry());
		let metadata = wrapper.get_metadata();
		assert_eq!(
            format!("{:?}", metadata),
            "FontMetadata { family: Fira Sans, style: normal, weight: 400, width: normal, codepoints: 1686 }"
        );
	}

	#[test]
	fn test_get_blocks() {
		let wrapper = FontWrapper::from(create_test_font_file_entry());
		let blocks = wrapper.get_blocks();
		assert_eq!(blocks.len(), 256);

		let mut list = blocks
			.iter()
			.map(|b| (b.start_index, b.glyphs.len()))
			.collect::<Vec<_>>();
		list.sort_unstable();

		assert_eq!(list.len(), 256);

		list.retain(|b| b.1 != 0);

		assert_eq!(
			list,
			[
				(0, 192),
				(256, 256),
				(512, 219),
				(768, 177),
				(1024, 240),
				(1280, 48),
				(3584, 1),
				(7424, 20),
				(7680, 157),
				(7936, 233),
				(8192, 67),
				(8448, 28),
				(8704, 16),
				(8960, 5),
				(9472, 2),
				(11264, 7),
				(42752, 14),
				(43776, 1),
				(64256, 2),
				(65024, 1)
			]
		);
	}
}
