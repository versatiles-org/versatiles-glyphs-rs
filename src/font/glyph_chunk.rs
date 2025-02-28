use super::entry::FontEntry;
use crate::{glyph::render_glyph, protobuf::PbfFontstack};
use anyhow::Result;
use prost::Message;
use std::{collections::HashMap, path::Path};

pub const CHUNK_SIZE: u32 = 256;

pub struct GlyphChunk<'a> {
	pub start_index: u32,
	pub glyphs: HashMap<u32, &'a FontEntry<'a>>,
}

impl GlyphChunk<'_> {
	pub fn render(&self, directory: &Path) -> Result<()> {
		let mut fontstack = PbfFontstack::default();
		for (index, font) in self.glyphs.iter() {
			if let Some(glyph) = render_glyph(&font.face, *index) {
				fontstack.glyphs.push(glyph);
			}
		}
		let mut out_buf = Vec::new();
		fontstack.encode(&mut out_buf)?;

		let filename = format!(
			"{}-{}.pbf",
			self.start_index,
			self.start_index + CHUNK_SIZE - 1
		);
		std::fs::write(directory.join(filename), out_buf)?;
		Ok(())
	}
}
