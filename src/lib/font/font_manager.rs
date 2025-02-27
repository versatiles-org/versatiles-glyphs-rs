use super::{load_font_metadata, metadata::FaceMetadata};
use crate::{glyph::render_glyph, protobuf::PbfFontstack};
use anyhow::{Context, Result};
use prost::Message;
use std::{collections::HashMap, path::Path, pin::Pin, slice};
use ttf_parser::Face;

struct FontEntry<'a> {
	data: Pin<Vec<u8>>,
	face: Face<'a>,
	metadata: FaceMetadata,
}

impl<'a> FontEntry<'a> {
	fn new(data: Vec<u8>) -> Result<Self> {
		unsafe {
			let data = Pin::new(data);
			let slice: &'a [u8] = slice::from_raw_parts(data.as_ptr(), data.len());
			let face = Face::parse(slice, 0).context("Could not parse font data")?;
			let metadata = load_font_metadata(&face)?;
			Ok(FontEntry {
				data,
				face,
				metadata,
			})
		}
	}
}

const CHUNK_SIZE: u32 = 256;

struct GlyphChunk<'a> {
	start_index: u32,
	glyphs: HashMap<u32, &'a FontEntry<'a>>,
}

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
			let mut fontstack = PbfFontstack::default();
			for (index, font) in chunk.glyphs.iter() {
				if let Some(glyph) = render_glyph(&font.face, *index) {
					fontstack.glyphs.push(glyph);
				}
				progress.inc(1);
			}
			let mut out_buf = Vec::new();
			fontstack.encode(&mut out_buf)?;

			let filename = format!(
				"{}-{}.pbf",
				chunk.start_index,
				chunk.start_index + CHUNK_SIZE - 1
			);
			std::fs::write(directory.join(filename), out_buf)?;
		}
		progress.finish();

		Ok(())
	}
}
