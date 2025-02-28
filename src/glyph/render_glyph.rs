use crate::{glyph::build_glyph_outline, protobuf::PbfGlyph, sdf::SdfGlyph};
use ttf_parser::Face;

/// Generate a PBF buffer of glyphs in [start..=end].
pub fn render_glyph(face: &Face, index: u32) -> Option<PbfGlyph> {
	let cp = char::from_u32(index).unwrap();

	// Check if face has a glyph for this codepoint
	let glyph_id = face.glyph_index(cp)?;

	assert!(cp as u32 == index, "Invalid codepoint: {}", index);

	let mut rings = build_glyph_outline(cp, face)?;

	let scale = 24.0 / face.units_per_em() as f32;
	rings.scale(scale);

	// Render the SDF
	if let Some(g) = SdfGlyph::from_rings(rings, 3, 0.25) {
		// Convert to your proto::Glyph
		let advance = face.glyph_hor_advance(glyph_id).unwrap() as f32 * scale;
		Some(g.into_pbf(index, advance.round() as u32))
	} else {
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const TEST_FONT: &[u8] = include_bytes!("../../testdata/Fira Sans - Regular.ttf");

	fn get_glyph(index: u32) -> PbfGlyph {
		let face = Face::parse(TEST_FONT, 0).unwrap();
		let glyph = render_glyph(&face, index).unwrap();
		assert_eq!(
			(glyph.width + 6) * (glyph.height + 6),
			glyph.bitmap.as_ref().unwrap().len() as u32
		);
		glyph
	}

	#[test]
	fn test_render_glyph_65() {
		let glyph = get_glyph(65);

		assert_eq!(glyph.width, 11);
		assert_eq!(glyph.height, 13);
		assert_eq!(glyph.left, 0);
		assert_eq!(glyph.top, 0);
		assert_eq!(glyph.advance, 11);
		assert_eq!(
			SdfGlyph::from_pbf(glyph).as_emoji_art(),
			vec![
				"        ░ ░ ▒ ▒ ▒ ▒ ▒ ░ ░ ░      ",
				"      ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░      ",
				"      ░ ░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░ ░      ",
				"      ░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▒ ▒ ░ ░    ",
				"    ░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░ ░    ",
				"    ░ ░ ▒ ▒ ▓ ▓ ▒ ▓ ▓ ▒ ▒ ░ ░    ",
				"    ░ ░ ▒ ▒ ▓ ▓ ▒ ▓ ▓ ▓ ▒ ░ ░    ",
				"  ░ ░ ▒ ▒ ▓ ▓ ▓ ▒ ▓ ▓ ▓ ▒ ▒ ░ ░  ",
				"  ░ ░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▓ ▓ ▒ ▒ ░ ░  ",
				"  ░ ░ ▒ ▒ ▓ ▓ ▒ ▒ ▒ ▓ ▓ ▒ ▒ ░ ░  ",
				"░ ░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░ ░",
				"░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░ ░",
				"░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▒ ▒ ░ ░ ░ ▒ ▒ ▓ ▓ ▒ ▒ ░",
				"░ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ░",
				"░ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░ ░ ░ ▒ ▒ ▒ ▒ ▒ ░",
				"░ ░ ░ ░ ░ ░ ░ ░   ░ ░ ░ ░ ░ ░ ░ ░"
			]
		);
	}

	#[test]
	fn test_render_glyph_230() {
		let glyph = get_glyph(230);

		assert_eq!(glyph.width, 15);
		assert_eq!(glyph.height, 11);
		assert_eq!(glyph.left, 0);
		assert_eq!(glyph.top, 0);
		assert_eq!(glyph.advance, 17);
		assert_eq!(
			SdfGlyph::from_pbf(glyph).as_emoji_art(),
			vec![
				"  ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░    ",
				"░ ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░ ░  ",
				"░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░ ░",
				"░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░ ░",
				"░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ▒ ░",
				"░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░",
				"░ ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▓ ▓ ▒ ▒ ░",
				"░ ░ ▒ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▓ ▒ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░",
				"░ ▒ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ▒ ░",
				"░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░",
				"  ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░ ░ ░",
				"    ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░    "
			]
		);
	}
	#[test]
	fn test_render_glyph_96() {
		let glyph = get_glyph(96);

		assert_eq!(glyph.width, 4);
		assert_eq!(glyph.height, 3);
		assert_eq!(glyph.left, 0);
		assert_eq!(glyph.top, 12);
		assert_eq!(glyph.advance, 6);
		assert_eq!(
			SdfGlyph::from_pbf(glyph).as_emoji_art(),
			vec![
				"░ ░ ░ ░ ░ ░ ░ ░    ",
				"░ ░ ▒ ▒ ▒ ▒ ░ ░ ░ ░",
				"░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▒ ▒",
				"░ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒",
				"░ ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ░",
				"░ ░ ░ ░ ░ ▒ ▒ ▒ ▒ ░",
				"      ░ ░ ░ ░ ░ ░ ░"
			]
		);
	}
}
