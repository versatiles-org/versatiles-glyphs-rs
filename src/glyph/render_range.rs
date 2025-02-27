use crate::protobuf::PbfFontstack;
use anyhow::{anyhow, Result};
use prost::Message;
use ttf_parser::Face;

use super::render_glyph;

/// Generate a PBF buffer of glyphs in [start..=end].
pub fn render_glyph_range(face: &Face, start: u32, end: u32) -> Result<Vec<u8>> {
	if end < start {
		return Err(anyhow!("start must be <= end"));
	}

	// Build a `FontStack` message from your .proto structures.
	// (The exact struct depends on your actual .proto definitions.)
	let mut fontstack = PbfFontstack::default();

	// Fill the name (like "FamilyName [style]" )
	// or just family name if no style is found.
	let family = face
		.names()
		.into_iter()
		.find_map(|n| n.to_string())
		.unwrap_or_else(|| "UnknownFamily".to_string());
	fontstack.name = family;
	fontstack.range = format!("{}-{}", start, end);

	// For codepoints in [start..end]
	for index in start..=end {
		// Render the SDF
		if let Some(glyph) = render_glyph(face, index) {
			fontstack.glyphs.push(glyph);
		}
	}

	// Now encode `fontstack` as a PBF with prost.
	let mut out_buf = Vec::new();
	fontstack.encode(&mut out_buf)?;
	Ok(out_buf)
}
