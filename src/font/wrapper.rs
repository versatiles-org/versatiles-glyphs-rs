//! Manages a collection of one or more [`FontFileEntry`] instances that represent a logical font.
//!
//! This wrapper abstracts multiple font files that share the same "family" identity
//! (for example, different languages). It provides methods to load font
//! data from file paths, retrieve metadata, and generate glyph blocks for rendering.

use super::{FontFileEntry, FontMetadata, GlyphBlock, GLYPH_BLOCK_SIZE};
use anyhow::{Context, Result};
use std::path::PathBuf;

/// A wrapper around one or more [`FontFileEntry`] instances.
/// Each [`FontWrapper`] is effectively a "logical" font that can span
/// multiple font files (e.g., for different languages).
#[derive(Debug, Default)]
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
	///
	/// A block is emitted for **every** 256-codepoint range of the Basic Multilingual
	/// Plane (`0-255` … `65280-65535`), even when this font contains no glyphs in that
	/// range. Mapbox GL / MapLibre request ranges on demand and treat a missing file
	/// (HTTP 404) as an error, logging a console warning per codepoint. Emitting an
	/// empty `.pbf` for uncovered ranges turns that 404 into a valid empty response,
	/// so the client silently falls back instead of warning. Codepoints outside the
	/// BMP (`> 0xFFFF`) are ignored, since the clients only render the BMP.
	pub fn get_blocks(&'a self) -> Vec<GlyphBlock<'a>> {
		/// Number of 256-codepoint blocks covering the Basic Multilingual Plane.
		const BMP_BLOCK_COUNT: u32 = 0x1_0000 / GLYPH_BLOCK_SIZE;

		// One block per BMP range, so even ranges this font doesn't cover are emitted.
		let mut blocks = (0..BMP_BLOCK_COUNT)
			.map(|i| GlyphBlock::new(i * GLYPH_BLOCK_SIZE))
			.collect::<Vec<GlyphBlock<'a>>>();

		// For each file, for each codepoint, place the codepoint into its corresponding block.
		for font_file in &self.files {
			for &codepoint in &font_file.metadata.codepoints {
				// Mapbox GL / MapLibre only render the Basic Multilingual Plane.
				if codepoint > 0xFFFF {
					continue;
				}
				let block_index = (codepoint / GLYPH_BLOCK_SIZE) as usize;
				let char_index = (codepoint % GLYPH_BLOCK_SIZE) as u8;
				blocks[block_index].set_glyph_font(char_index, font_file);
			}
		}

		blocks
	}

	/// Returns the [`FontMetadata`] of the first font file in this wrapper.
	///
	/// # Errors
	///
	/// Returns an error if this wrapper has no files.
	pub fn get_metadata(&self) -> Result<&FontMetadata> {
		Ok(&self
			.files
			.first()
			.context("FontWrapper has no files")?
			.metadata)
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
		let metadata = wrapper.get_metadata().unwrap();
		assert_eq!(
            format!("{metadata:?}", ),
            "FontMetadata { family: Fira Sans, style: normal, weight: 400, width: normal, codepoints: 1686 }"
        );
	}

	#[test]
	fn test_get_metadata_empty_wrapper_errors() {
		let wrapper = FontWrapper::default();
		let err = wrapper.get_metadata().unwrap_err();
		assert!(err.to_string().contains("FontWrapper has no files"));
	}

	#[test]
	fn test_add_paths_missing_file_errors() {
		let mut wrapper = FontWrapper::default();
		let err = wrapper
			.add_paths(&[PathBuf::from("/nonexistent.ttf")])
			.unwrap_err();
		assert!(err.to_string().contains("reading font file"));
	}

	#[test]
	fn test_add_paths_invalid_font_errors() {
		// Create a temp file with garbage bytes that won't parse as a font.
		let dir = tempfile::tempdir().unwrap();
		let bad = dir.path().join("garbage.ttf");
		std::fs::write(&bad, b"not a font").unwrap();

		let mut wrapper = FontWrapper::default();
		let err = wrapper.add_paths(&[bad]).unwrap_err();
		assert!(err.to_string().contains("Could not parse font data"));
	}

	#[test]
	fn test_try_from_paths_creates_wrapper() {
		let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/Fira Sans - Regular.ttf");
		let wrapper = FontWrapper::try_from(&[path][..]).unwrap();
		assert_eq!(wrapper.files.len(), 1);
		assert_eq!(wrapper.get_metadata().unwrap().family, "Fira Sans");
	}

	#[test]
	fn test_try_from_paths_propagates_errors() {
		let err = FontWrapper::try_from(&[PathBuf::from("/nonexistent.ttf")][..]).unwrap_err();
		assert!(err.to_string().contains("reading font file"));
	}

	#[test]
	fn test_get_blocks() {
		let wrapper = FontWrapper::from(create_test_font_file_entry());
		let blocks = wrapper.get_blocks();

		let mut list = blocks
			.iter()
			.map(|b| (b.start_index, b.glyphs.len()))
			.collect::<Vec<_>>();
		list.sort_unstable();

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
