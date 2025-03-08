use super::RingBuilder;
use crate::{protobuf::PbfGlyph, render::RendererTrait};
use ttf_parser::Face;

/// Generate a PBF buffer of glyphs in [start..=end].
pub fn render_glyph(face: &Face, index: u32, renderer: &impl RendererTrait) -> Option<PbfGlyph> {
	let cp = char::from_u32(index).unwrap();

	let glyph_id = face.glyph_index(cp)?;
	let mut builder = RingBuilder::default();
	face.outline_glyph(glyph_id, &mut builder);
	let mut rings = builder.into_rings();

	let scale = 24.0 / face.units_per_em() as f64;
	rings.scale(scale);

	let advance = (face.glyph_hor_advance(glyph_id).unwrap() as f64 * scale).round() as u32;

	// Render the SDF
	let sdf_option = renderer.render(rings);
	Some(if let Some(mut sdf) = sdf_option {
		sdf.y1 -= 24;
		sdf.into_pbf_glyph(index, advance)
	} else {
		PbfGlyph::empty(index, advance)
	})
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{render::RendererPrecise, utils::bitmap_as_ascii_art};

	const TEST_FONT: &[u8] = include_bytes!("../../testdata/Fira Sans - Regular.ttf");

	fn get_glyph(index: u32) -> PbfGlyph {
		let face = Face::parse(TEST_FONT, 0).unwrap();
		let glyph = render_glyph(&face, index, &RendererPrecise {}).unwrap();

		if let Some(bitmap) = &glyph.bitmap {
			assert_eq!(bitmap.len() as u32, (glyph.width + 6) * (glyph.height + 6));
		}

		glyph
	}

	fn as_art(glyph: &PbfGlyph) -> Vec<String> {
		bitmap_as_ascii_art(glyph.bitmap.as_ref().unwrap(), glyph.width as usize + 6)
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

		assert_eq!(glyph.width, 14);
		assert_eq!(glyph.height, 17);
		assert_eq!(glyph.left, 0);
		assert_eq!(glyph.top, -7);
		assert_eq!(glyph.advance, 14);
		assert_eq!(
			as_art(&glyph),
			[
				"            ░░░░░░░░░░░░░░░░            ",
				"          ░░░░▒▒▒▒▒▒▒▒▒▒░░░░░░          ",
				"        ░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░          ",
				"        ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒░░░░        ",
				"        ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒░░░░        ",
				"        ░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░        ",
				"      ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░      ",
				"      ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░      ",
				"      ░░░░▒▒▓▓▓▓▓▓▒▒▓▓▓▓▓▓▒▒▒▒░░░░      ",
				"    ░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░    ",
				"    ░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░    ",
				"    ░░░░▒▒▒▒▓▓▓▓▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░    ",
				"  ░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░    ",
				"  ░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░  ",
				"  ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░  ",
				"░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒░░░░  ",
				"░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░",
				"░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░",
				"░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░",
				"░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░",
				"░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░",
				"░░░░▒▒▒▒▒▒▒▒▒▒░░░░  ░░░░░░▒▒▒▒▒▒▒▒▒▒░░░░",
				"░░░░░░░░░░░░░░░░░░    ░░░░░░░░░░░░░░░░░░"
			]
		);
	}

	#[test]
	fn test_render_glyph_230() {
		let glyph = get_glyph(230);

		assert_eq!(glyph.width, 19);
		assert_eq!(glyph.height, 14);
		assert_eq!(glyph.left, 1);
		assert_eq!(glyph.top, -11);
		assert_eq!(glyph.advance, 20);
		assert_eq!(
			as_art(&glyph),
			[
				"    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░        ",
				"  ░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░▒▒▒▒▒▒▒▒▒▒▒▒░░░░░░      ",
				"░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░    ",
				"░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒░░░░  ",
				"░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░  ",
				"░░░░▒▒▒▒▓▓▓▓▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒░░░░",
				"  ░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░",
				"  ░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▓▓▓▓▒▒▒▒░░░░",
				"░░░░▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░",
				"░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░",
				"░░▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░",
				"░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░",
				"░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░░░",
				"░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▓▓▒▒▒▒▒▒░░░░",
				"░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░",
				"░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒░░░░",
				"░░░░▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒░░░░  ",
				"  ░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░░░░░  ",
				"    ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░      ",
				"      ░░░░░░░░░░░░░░░░  ░░░░░░░░░░░░░░░░░░        "
			]
		);
	}
	#[test]
	fn test_render_glyph_96() {
		let glyph = get_glyph(96);

		assert_eq!(glyph.width, 7);
		assert_eq!(glyph.height, 5);
		assert_eq!(glyph.left, 0);
		assert_eq!(glyph.top, -4);
		assert_eq!(glyph.advance, 7);
		assert_eq!(
			as_art(&glyph),
			[
				"    ░░░░░░░░░░            ",
				"  ░░░░░░░░░░░░░░░░        ",
				"  ░░░░▒▒▒▒▒▒▒▒░░░░░░░░    ",
				"░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░░░  ",
				"░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒░░░░░░",
				"░░░░▒▒▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒░░░░",
				"░░░░▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░",
				"░░░░░░▒▒▒▒▒▒▒▒▓▓▓▓▒▒▒▒░░░░",
				"  ░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒░░░░",
				"    ░░░░░░░░░░▒▒▒▒▒▒░░░░  ",
				"          ░░░░░░░░░░░░░░  "
			]
		);
	}
}
