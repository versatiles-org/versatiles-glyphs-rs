use super::{fontstack::Fontstack, glyph::Glyph, glyphs::Glyphs};
use anyhow::Result;
use prost::Message;

pub struct PbfGlyphs {
	fontstack: Fontstack,
}

impl PbfGlyphs {
	pub fn new(name: String, range: String) -> Self {
		Self {
			fontstack: Fontstack::new(name, range),
		}
	}

	pub fn push(&mut self, glyph: Glyph) {
		self.fontstack.glyphs.push(glyph);
	}

	pub fn into_vec(self) -> Result<Vec<u8>> {
		let glyphs = Glyphs {
			stacks: vec![self.fontstack],
		};
		let mut out_buf = Vec::new();
		glyphs.encode(&mut out_buf)?;
		Ok(out_buf)
	}
}
