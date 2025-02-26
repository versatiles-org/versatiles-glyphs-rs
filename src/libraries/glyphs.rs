use super::{
	protobuf::{PbfFontstack, PbfGlyph},
	sdf::render_sdf,
};
use anyhow::{anyhow, Result};
use prost::Message;
use ttf_parser::Face;

/// Generate a PBF buffer of glyphs in [start..=end].
pub fn range_glyphs(face: &Face, start: char, end: char) -> Result<Vec<u8>> {
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
	for cp in start..=end {
		// Check if face has a glyph for this codepoint
		if face.glyph_index(cp).is_none() {
			continue;
		}
		// Render the SDF
		if let Some(g) = render_sdf(cp, &face, 3, 0.25) {
			// Convert to your proto::Glyph
			let glyph = PbfGlyph {
				id: cp as u32,
				bitmap: Some(g.bitmap), // raw bytes
				width: g.width,
				height: g.height,
				left: g.left,
				top: g.top,
				advance: g.advance as u32,
			};
			fontstack.glyphs.push(glyph);
		}
	}

	// Now encode `fontstack` as a PBF with prost.
	let mut out_buf = Vec::new();
	fontstack.encode(&mut out_buf)?;
	Ok(out_buf)
}
