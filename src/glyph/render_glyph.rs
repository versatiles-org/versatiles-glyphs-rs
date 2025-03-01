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

	let advance = (face.glyph_hor_advance(glyph_id).unwrap() as f32 * scale).round() as u32;

	// Render the SDF
	let sdf_option = SdfGlyph::from_rings(rings, 3, 0.25);
	Some(if let Some(sdf) = sdf_option {
		PbfGlyph::from_sdf(sdf, index, advance)
	} else {
		PbfGlyph::empty(index, advance)
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	const TEST_FONT: &[u8] = include_bytes!("../../testdata/Fira Sans - Regular.ttf");

	fn get_glyph(index: u32) -> PbfGlyph {
		let face = Face::parse(TEST_FONT, 0).unwrap();
		let glyph = render_glyph(&face, index).unwrap();

		if let Some(bitmap) = &glyph.bitmap {
			assert_eq!(bitmap.len() as u32, (glyph.width + 6) * (glyph.height + 6));
		}

		glyph
	}

	#[test]
	fn test_render_glyph_32() {
		let glyph = get_glyph(32);

		assert_eq!(glyph.width, 0);
		assert_eq!(glyph.height, 0);
		assert_eq!(glyph.left, 0);
		assert_eq!(glyph.top, 0);
		assert_eq!(glyph.advance, 6);
		assert!(glyph.bitmap.is_none());
	}

	#[test]
	fn test_render_glyph_65() {
		let glyph = get_glyph(65);

		assert_eq!(glyph.width, 13);
		assert_eq!(glyph.height, 16);
		assert_eq!(glyph.left, 0);
		assert_eq!(glyph.top, -7);
		assert_eq!(glyph.advance, 14);
		assert_eq!(
			SdfGlyph::from_pbf(glyph).as_emoji_art(),
			vec![
				"          ░ ░ ▒ ▒ ▒ ▒ ▒ ░ ░ ░        ",
				"        ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░        ",
				"        ░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▒ ▒ ░        ",
				"        ░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▒ ▒ ░ ░      ",
				"      ░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░ ░      ",
				"      ░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░ ░      ",
				"      ░ ░ ▒ ▒ ▓ ▓ ▒ ▓ ▓ ▓ ▒ ▒ ░ ░    ",
				"    ░ ░ ▒ ▒ ▓ ▓ ▓ ▒ ▓ ▓ ▓ ▒ ▒ ░ ░    ",
				"    ░ ░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▓ ▓ ▒ ▒ ░ ░    ",
				"    ░ ░ ▒ ▒ ▓ ▓ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░ ░  ",
				"  ░ ░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░ ░  ",
				"  ░ ░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░ ░  ",
				"  ░ ░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░ ░",
				"░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░ ░",
				"░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░ ░",
				"░ ░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░ ░ ░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░ ░ ░ ░ ▒ ▒ ▓ ▓ ▓ ▒ ▒",
				"░ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░   ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒",
				"░ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░   ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ░",
				"░ ░ ░ ░ ░ ░ ░ ░ ░     ░ ░ ░ ░ ░ ░ ░ ░"
			]
		);
	}

	#[test]
	fn test_render_glyph_230() {
		let glyph = get_glyph(230);

		assert_eq!(glyph.width, 18);
		assert_eq!(glyph.height, 13);
		assert_eq!(glyph.left, 1);
		assert_eq!(glyph.top, -11);
		assert_eq!(glyph.advance, 20);
		assert_eq!(
			SdfGlyph::from_pbf(glyph).as_emoji_art(),
			vec![
				"  ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░      ",
				"  ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░ ░  ",
				"░ ░ ▒ ▒ ▒ ▒ ▒ ▓ ▓ ▒ ▒ ▒ ▒ ▒ ▒ ▓ ▓ ▒ ▒ ▒ ▒ ░ ░  ",
				"░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ▒ ░ ░",
				"░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░ ░",
				"░ ░ ▒ ▒ ▓ ▒ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░",
				"  ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░",
				"░ ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ░",
				"░ ░ ▒ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░",
				"░ ▒ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▓ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▒ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ▒ ▓ ▓ ▒ ▒ ▒ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ░",
				"░ ░ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▓ ▓ ▒ ▒ ▒ ░",
				"░ ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░ ░",
				"  ░ ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░ ░ ░  ",
				"    ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░ ░    "
			]
		);
	}
	#[test]
	fn test_render_glyph_96() {
		let glyph = get_glyph(96);

		assert_eq!(glyph.width, 5);
		assert_eq!(glyph.height, 3);
		assert_eq!(glyph.left, 0);
		assert_eq!(glyph.top, -4);
		assert_eq!(glyph.advance, 7);
		assert_eq!(
			SdfGlyph::from_pbf(glyph).as_emoji_art(),
			vec![
				"░ ░ ▒ ▒ ▒ ▒ ░ ░ ░    ",
				"░ ░ ▒ ▒ ▒ ▒ ▒ ░ ░ ░ ░",
				"░ ▒ ▒ ▓ ▓ ▒ ▒ ▒ ▒ ░ ░",
				"░ ▒ ▒ ▓ ▓ ▓ ▓ ▒ ▒ ▒ ▒",
				"░ ▒ ▒ ▒ ▓ ▓ ▓ ▓ ▓ ▒ ▒",
				"░ ▒ ▒ ▒ ▒ ▒ ▓ ▓ ▓ ▒ ▒",
				"░ ░ ░ ▒ ▒ ▒ ▒ ▒ ▒ ▒ ░",
				"  ░ ░ ░ ░ ░ ▒ ▒ ▒ ▒ ░",
				"      ░ ░ ░ ░ ░ ░ ░ ░"
			]
		);
	}
}
