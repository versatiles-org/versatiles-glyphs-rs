use super::{
	entry::FontEntry,
	glyph_chunk::{GlyphChunk, CHUNK_SIZE},
};
use anyhow::{Ok, Result};
use std::{collections::HashMap, path::Path};

#[derive(Default)]
pub struct FontManager<'a> {
	fonts: Vec<FontEntry<'a>>,
}

impl<'a> FontManager<'a> {
	pub fn add_font(&mut self, data: Vec<u8>) -> Result<()> {
		self.fonts.push(FontEntry::new(data)?);
		Ok(())
	}

	fn get_chunks(&self) -> Vec<GlyphChunk> {
		let mut chunks = HashMap::<u32, GlyphChunk<'a>>::new();
		for font in self.fonts.iter() {
			for codepoint in &font.metadata.codepoints {
				let chunk = codepoint / CHUNK_SIZE;
				let index = codepoint % CHUNK_SIZE;
				let entry = chunks.entry(chunk).or_insert(GlyphChunk {
					start_index: chunk * CHUNK_SIZE,
					glyphs: HashMap::new(),
				});
				entry.glyphs.entry(index).or_insert(font);
			}
		}
		chunks.into_values().collect()
	}

	pub fn render_glyphs(&self, directory: &Path) -> Result<()> {
		let chunks = self.get_chunks();

		let sum = chunks.iter().map(|chunk| chunk.glyphs.len() as u64).sum();
		let progress = indicatif::ProgressBar::new(sum);
		progress.set_position(0);

		for chunk in chunks.iter() {
			chunk.render(directory)?;
			progress.inc(chunk.glyphs.len() as u64);
		}
		progress.finish();

		Ok(())
	}
}
