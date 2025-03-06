use super::{FontFileEntry, FontMetadata, GlyphBlock, GLYPH_BLOCK_SIZE};
use anyhow::{Context, Result};
use std::{collections::HashMap, path::PathBuf};

#[derive(Default)]
pub struct FontWrapper<'a> {
	files: Vec<FontFileEntry<'a>>,
}

impl<'a> FontWrapper<'a> {
	pub fn add_file(&mut self, file: FontFileEntry<'a>) {
		self.files.push(file);
	}

	pub fn add_paths(&mut self, sources: &[PathBuf]) -> Result<()> {
		for path in sources {
			let data =
				std::fs::read(path).with_context(|| format!("reading font file \"{path:?}\""))?;
			self.files.push(FontFileEntry::new(data)?);
		}
		Ok(())
	}

	pub fn get_blocks(&'a self) -> Vec<GlyphBlock<'a>> {
		let mut blocks = HashMap::<u32, GlyphBlock<'a>>::new();
		for font in self.files.iter() {
			for codepoint in &font.metadata.codepoints {
				let block_index = codepoint / GLYPH_BLOCK_SIZE;
				let char_index = (codepoint % GLYPH_BLOCK_SIZE) as u8;
				let block = blocks
					.entry(block_index)
					.or_insert(GlyphBlock::new(block_index * GLYPH_BLOCK_SIZE));
				block.set_glyph_font(char_index, font);
			}
		}
		blocks.into_values().collect()
	}

	pub fn get_metadata(&self) -> &FontMetadata {
		&self.files.first().unwrap().metadata
	}
}

impl<'a> From<FontFileEntry<'a>> for FontWrapper<'a> {
	fn from(file: FontFileEntry<'a>) -> Self {
		let mut font = FontWrapper::default();
		font.add_file(file);
		font
	}
}

impl TryFrom<&[PathBuf]> for FontWrapper<'_> {
	type Error = anyhow::Error;

	fn try_from(paths: &[PathBuf]) -> Result<Self> {
		let mut font = FontWrapper::default();
		font.add_paths(paths)?;
		Ok(font)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	// Helper function to create a FontFileEntry from a valid test font.
	// Adjust the relative path to point to your valid test font file.
	fn create_test_font_file_entry<'a>() -> FontFileEntry<'a> {
		FontFileEntry::new(include_bytes!("../../testdata/Fira Sans - Regular.ttf").to_vec()).unwrap()
	}

	#[test]
	fn test_add_file_and_get_metadata() {
		let wrapper = FontWrapper::from(create_test_font_file_entry());
		let metadata = wrapper.get_metadata();
		assert_eq!(format!("{:?}", metadata), "FontMetadata { family: Fira Sans, style: normal, weight: 400, width: normal, codepoints: 1686 }");
	}

	#[test]
	fn test_get_blocks() {
		let wrapper = FontWrapper::from(create_test_font_file_entry());
		let blocks = wrapper.get_blocks();
		assert_eq!(blocks.len(), 20);

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
