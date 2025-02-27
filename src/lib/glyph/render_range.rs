use crate::{
	glyph::build_glyph_outline,
	protobuf::{PbfFontstack, PbfGlyph},
	sdf::render_sdf,
};
use anyhow::{anyhow, Result};
use prost::Message;
use ttf_parser::Face;

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

/// Generate a PBF buffer of glyphs in [start..=end].
pub fn render_glyph(face: &Face, index: u32) -> Option<PbfGlyph> {
	let cp = char::from_u32(index).unwrap();

	// Check if face has a glyph for this codepoint
	let glyph_id = face.glyph_index(cp)?;

	assert!(cp as u32 == index, "Invalid codepoint: {}", index);

	let rings = build_glyph_outline(cp, face, 24.0)?;

	// Render the SDF
	if let Some(g) = render_sdf(rings, 3, 0.25) {
		// Convert to your proto::Glyph
		Some(PbfGlyph {
			id: index,
			bitmap: Some(g.bitmap), // raw bytes
			width: g.width,
			height: g.height,
			left: g.left,
			top: g.top,
			advance: face.glyph_hor_advance(glyph_id).unwrap() as u32,
		})
	} else {
		None
	}
}
