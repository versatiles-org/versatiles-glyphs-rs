use super::font_file_entry::FontFileEntry;
use crate::{glyph::render_glyph, protobuf::PbfGlyphs};
use anyhow::Result;
use std::{collections::HashMap, path::Path};

pub const CHARACTER_BLOCK_SIZE: u32 = 256;

/// A block of up to 256 characters that can be rendered to a PBF file.
pub struct CharacterBlock<'a> {
	start_index: u32,
	characters: HashMap<u8, &'a FontFileEntry<'a>>,
}

impl<'a> CharacterBlock<'a> {
	pub fn new(start_index: u32) -> CharacterBlock<'a> {
		CharacterBlock {
			start_index,
			characters: HashMap::new(),
		}
	}

	pub fn set_char_font(&mut self, char_index: u8, font: &'a FontFileEntry<'a>) {
		self.characters.entry(char_index).or_insert(font);
	}

	pub fn len(&self) -> usize {
		self.characters.len()
	}

	fn range(&self) -> String {
		format!(
			"{}-{}",
			self.start_index,
			self.start_index + CHARACTER_BLOCK_SIZE - 1
		)
	}

	pub fn render(&self, font_name: String) -> Result<Vec<u8>> {
		let mut glyphs = PbfGlyphs::new(font_name, self.range());

		for (index, font) in self.characters.iter() {
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

	pub fn render_to_file(&self, directory: &Path) -> Result<()> {
		let filename = directory.join(self.filename());
		let basename = directory
			.components()
			.last()
			.unwrap()
			.as_os_str()
			.to_os_string()
			.into_string()
			.unwrap();

		let out_buf = self.render(basename)?;

		std::fs::write(filename, out_buf)?;
		Ok(())
	}
}
