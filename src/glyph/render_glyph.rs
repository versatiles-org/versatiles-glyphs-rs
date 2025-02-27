use crate::{glyph::build_glyph_outline, protobuf::PbfGlyph, sdf::render_sdf};
use ttf_parser::Face;

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
