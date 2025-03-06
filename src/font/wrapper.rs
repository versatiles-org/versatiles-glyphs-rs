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
