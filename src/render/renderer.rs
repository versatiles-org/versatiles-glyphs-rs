use super::{
	renderer_dummy::renderer_dummy, renderer_precise::renderer_precise, ring_builder::RingBuilder,
	RenderResult, BUFFER,
};
use crate::{geometry::Rings, protobuf::PbfGlyph};
use ttf_parser::Face;

#[derive(Debug, Clone)]
enum RendererMode {
	Precise,
	Dummy,
}

#[derive(Debug, Clone)]
/// A renderer for creating signed distance fields (SDF) from glyph outlines.
pub struct Renderer {
	mode: RendererMode,
}

impl Renderer {
	/// Creates a new renderer with the specified mode.
	pub fn new(dummy: bool) -> Self {
		if dummy {
			Renderer::new_dummy()
		} else {
			Renderer::new_precise()
		}
	}
	/// Creates a new renderer with the precise mode.
	pub fn new_precise() -> Self {
		Renderer {
			mode: RendererMode::Precise,
		}
	}
	/// Creates a new renderer with the dummy mode. This mode generates empty bitmaps and is used for testing.
	pub fn new_dummy() -> Self {
		Renderer {
			mode: RendererMode::Dummy,
		}
	}

	/// Prepares the geometry and compute bounding box data for rendering.
	///
	/// This method:
	/// - Computes the bounding box for the given `rings`.
	/// - Adjusts it by adding a `BUFFER` on all sides.
	/// - Translates the outline to ensure it starts at `(0, 0)`.
	/// - Produces a [`RenderResult`] with the computed width, height,
	///   and coordinate offsets.
	///
	/// Returns [`None`] if the bounding box is empty (e.g., no outline data).
	fn prepare_glyph(&self, rings: &Rings) -> Option<RenderResult> {
		// Calculate the real glyph bbox.
		let bbox = rings.get_bbox();

		if bbox.is_empty() {
			return None;
		}

		let x0 = bbox.min.x.floor() as i32 - BUFFER;
		let y0 = bbox.min.y.floor() as i32 - BUFFER;
		let x1 = bbox.max.x.ceil() as i32 + BUFFER;
		let y1 = bbox.max.y.ceil() as i32 + BUFFER;
		let width = (x1 - x0) as usize;
		let height = (y1 - y0) as usize;

		let glyph = RenderResult {
			x0,
			y1,
			x1,
			y0,
			width: width as u32,
			height: height as u32,
			bitmap: None,
		};

		Some(glyph)
	}

	/// Renders a single glyph to a [`PbfGlyph`], given a font [`Face`],
	/// a Unicode `index` (`char::from_u32`) and a rendering backend (`renderer`).
	///
	/// This process outlines the glyph, scales it, and uses the provided renderer
	/// to create a signed distance field (SDF). The SDF is then converted
	/// into a [`PbfGlyph`]. If no SDF is produced, an empty glyph is returned.
	///
	/// # Return
	///
	/// Returns [`None`] if no corresponding glyph index can be found in `face`.
	pub fn render_glyph(&self, face: &Face, index: u32) -> Option<PbfGlyph> {
		let cp = char::from_u32(index).unwrap();

		let glyph_id = face.glyph_index(cp)?;
		let scale = 24.0 / face.units_per_em() as f64;

		let mut builder = RingBuilder::default();
		face.outline_glyph(glyph_id, &mut builder);
		let mut rings = builder.into_rings();

		let advance = (face.glyph_hor_advance(glyph_id).unwrap() as f64 * scale).round() as u32;

		if rings.is_empty() {
			return Some(PbfGlyph::empty(index, advance));
		}

		rings.scale(scale);

		let mut glyph = if let Some(g) = self.prepare_glyph(&rings) {
			g
		} else {
			return Some(PbfGlyph::empty(index, advance));
		};

		// Render the SDF
		match self.mode {
			RendererMode::Precise => renderer_precise(&mut glyph, rings),
			RendererMode::Dummy => renderer_dummy(&mut glyph),
		}

		// Shift the SDF output to re-base the glyph
		glyph.y1 -= 24;

		Some(glyph.into_pbf_glyph(index, advance))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::utils::bitmap_as_ascii_art;

	const TEST_FONT: &[u8] = include_bytes!("../../testdata/Fira Sans - Regular.ttf");

	fn get_glyph(index: u32) -> PbfGlyph {
		let face = Face::parse(TEST_FONT, 0).unwrap();
		let renderer = Renderer::new_precise();
		let glyph = renderer.render_glyph(&face, index).unwrap();

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
