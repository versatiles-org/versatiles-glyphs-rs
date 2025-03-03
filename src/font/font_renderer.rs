use super::{
	character_block::{CharacterBlock, CHARACTER_BLOCK_SIZE},
	font_file_entry::FontFileEntry,
};
use anyhow::{Context, Ok, Result};
use indicatif::ProgressStyle;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
	collections::HashMap,
	fs::create_dir_all,
	path::{Path, PathBuf},
};

#[derive(Default)]
pub struct FontRenderer<'a> {
	fonts: Vec<FontFileEntry<'a>>,
}

impl<'a> FontRenderer<'a> {
	pub fn from_filenames(filenames: Vec<&str>) -> Result<Self> {
		let mut font = FontRenderer::default();
		for filename in filenames {
			font.add_font_file(Path::new(filename))?;
		}
		Ok(font)
	}

	pub fn from_paths(paths: Vec<PathBuf>) -> Result<Self> {
		let mut font = FontRenderer::default();
		for path in paths {
			font.add_font_file(&path)?;
		}
		Ok(font)
	}

	pub fn add_font_data(&mut self, data: Vec<u8>) -> Result<()> {
		self.fonts.push(FontFileEntry::new(data)?);
		Ok(())
	}

	pub fn add_font_file(&mut self, path: &Path) -> Result<()> {
		self.add_font_data(
			std::fs::read(path).with_context(|| format!("reading font file \"{path:?}\""))?,
		)
	}

	pub fn get_chunks(&'a self) -> Vec<CharacterBlock<'a>> {
		let mut blocks = HashMap::<u32, CharacterBlock<'a>>::new();
		for font in self.fonts.iter() {
			for codepoint in &font.metadata.codepoints {
				let block_index = codepoint / CHARACTER_BLOCK_SIZE;
				let char_index = (codepoint % CHARACTER_BLOCK_SIZE) as u8;
				let block = blocks
					.entry(block_index)
					.or_insert(CharacterBlock::new(block_index * CHARACTER_BLOCK_SIZE));
				block.set_char_font(char_index, font);
			}
		}
		blocks.into_values().collect()
	}

	pub fn render_glyphs(&self, directory: &Path) -> Result<()> {
		create_dir_all(directory).with_context(|| format!("creating directory \"{directory:?}\""))?;

		let chunks = self.get_chunks();

		let sum = chunks.iter().map(|chunk| chunk.len() as u64).sum();
		let progress = indicatif::ProgressBar::new(sum)
			.with_style(ProgressStyle::with_template(
				"[{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} {msg}",
			)?)
			.with_position(0);

		chunks.par_iter().for_each(|chunk| {
			chunk.render_to_file(directory).unwrap();
			progress.inc(chunk.len() as u64);
		});

		progress.finish();

		Ok(())
	}
}
