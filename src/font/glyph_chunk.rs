use super::entry::FontEntry;
use crate::{glyph::render_glyph, protobuf::{PbfFontstack, PbfGlyphs}};
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

		let range = format!("{}-{}", self.start_index, self.start_index + CHUNK_SIZE - 1);
		let filename = directory.join(format!("{range}.pbf"));
		let basename = directory
			.components()
			.last()
			.unwrap()
			.as_os_str()
			.to_os_string()
			.into_string()
			.unwrap();

		fontstack.name = basename;
		fontstack.range = range;

		for (index, font) in self.glyphs.iter() {
			if let Some(glyph) = render_glyph(&font.face, *index) {
				fontstack.glyphs.push(glyph);
			}
		}
		let mut out_buf = Vec::new();
		let glyphs = PbfGlyphs {
			stacks: vec![fontstack],
		};
		glyphs.encode(&mut out_buf)?;

		std::fs::write(filename, out_buf)?;
		Ok(())
	}
}
