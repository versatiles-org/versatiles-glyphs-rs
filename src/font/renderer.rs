use super::{GlyphBlock, FontFileEntry, FontMetadata, GLYPH_BLOCK_SIZE};
use anyhow::{Context, Ok, Result};
use std::{
	collections::HashMap,
	path::{Path, PathBuf},
};

#[derive(Default)]
pub struct FontRenderer<'a> {
	fonts: Vec<FontFileEntry<'a>>,
}

impl<'a> FontRenderer<'a> {
	pub fn from_paths(paths: &Vec<PathBuf>) -> Result<Self> {
		let mut font = FontRenderer::default();
		for path in paths {
			font.add_font_path(path)?;
		}
		Ok(font)
	}

	pub fn add_font(&mut self, font: FontFileEntry<'a>) {
		self.fonts.push(font);
	}

	pub fn add_font_data(&mut self, data: Vec<u8>) -> Result<()> {
		self.fonts.push(FontFileEntry::new(data)?);
		Ok(())
	}

	pub fn add_font_path(&mut self, path: &Path) -> Result<()> {
		self.add_font_data(
			std::fs::read(path).with_context(|| format!("reading font file \"{path:?}\""))?,
		)
	}

	pub fn add_font_paths(&mut self, sources: &Vec<PathBuf>) -> Result<()> {
		for source in sources {
			self.add_font_path(source)?;
		}
		Ok(())
	}

	pub fn get_blocks(&'a self) -> Vec<GlyphBlock<'a>> {
		let mut blocks = HashMap::<u32, GlyphBlock<'a>>::new();
		for font in self.fonts.iter() {
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
		&self.fonts.first().unwrap().metadata
	}
}
