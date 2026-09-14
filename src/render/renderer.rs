use super::{
	renderer_dummy::renderer_dummy, renderer_precise::renderer_precise, ring_builder::RingBuilder,
	RenderResult, BUFFER, GLYPH_SIZE,
};
use crate::{
	geometry::{Point, Rings},
	protobuf::PbfGlyph,
};
use anyhow::{ensure, Result};
use ttf_parser::Face;

/// The largest allowed SDF quantization step, see [`Renderer::with_sdf_step`].
pub const MAX_SDF_STEP: u8 = 64;

#[derive(Debug, Clone)]
enum RendererMode {
	Precise,
	Dummy,
}

#[derive(Debug, Clone)]
/// A renderer for creating signed distance fields (SDF) from glyph outlines.
pub struct Renderer {
	mode: RendererMode,
	sdf_step: u8,
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
			sdf_step: 1,
		}
	}
	/// Creates a new renderer with the dummy mode. This mode generates empty bitmaps and is used for testing.
	pub fn new_dummy() -> Self {
		Renderer {
			mode: RendererMode::Dummy,
			sdf_step: 1,
		}
	}

	/// Sets the quantization step for SDF values.
	///
	/// Every SDF value is rounded to the nearest multiple of `step` (capped at 255),
	/// which makes the bitmaps compress much better at the cost of precision:
	/// with a radius of 8 px, one byte step is 1/32 px, so a step of 4 means 1/8 px.
	/// `0` stays `0`, and `192` (exactly on the outline) and `255` remain reachable.
	/// A step of `1` (the default) keeps the output unchanged.
	///
	/// # Errors
	///
	/// Returns an error unless `step` is a power of two from 1 to [`MAX_SDF_STEP`].
	pub fn with_sdf_step(mut self, step: u8) -> Result<Self> {
		ensure!(
			step.is_power_of_two() && step <= MAX_SDF_STEP,
			"SDF step must be a power of two from 1 to {MAX_SDF_STEP}, but is {step}"
		);
		self.sdf_step = step;
		Ok(self)
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
	///
	/// # Bbox rounding
	///
	/// The float bbox is converted to integer pixel bounds with `floor` on
	/// `min` and `ceil` on `max`, so the integer cell always *contains* the
	/// float bbox. The trade-off is that the actual outline can sit up to 1
	/// pixel inside each edge — see the [module-level docs](super) for the
	/// full discussion of this rounding artifact and why `BUFFER` is only
	/// 3 pixels even though the SDF gradient extends to 8.
	fn prepare_glyph(&self, rings: &Rings) -> Option<RenderResult> {
		let bbox = rings.get_bbox();

		if bbox.is_empty() {
			return None;
		}

		// floor/ceil + BUFFER: the bitmap's content area is the integer cell
		// containing `bbox`, padded by BUFFER pixels on every side for the SDF.
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
		let cp = char::from_u32(index)?;

		let glyph_id = face.glyph_index(cp)?;
		let scale = GLYPH_SIZE as f64 / face.units_per_em() as f64;

		let mut builder = RingBuilder::default();
		face.outline_glyph(glyph_id, &mut builder);
		let mut rings = builder.into_rings();

		// `* 0.95` matches the empirical scale used by other Mapbox-spec glyph
		// pipelines (e.g. fontnik) so renderings line up with existing tiles.
		let advance_float = face.glyph_hor_advance(glyph_id).unwrap_or(0) as f64 * scale * 0.95;
		let advance = advance_float.round() as u32;

		if rings.is_empty() {
			return Some(PbfGlyph::empty(index, advance));
		}

		rings.scale(scale);

		// `advance` in the PBF must be an integer, but `advance_float` rarely
		// is. We absorb half the rounding error by translating the outline by
		// `dx` (≤ ±0.25 px) so it stays visually centered inside the integer
		// advance cell. This sub-pixel shift is what makes the outline land at
		// non-integer positions and feeds into the bbox rounding artifact
		// described in `prepare_glyph` below.
		let dx = (advance as f64 - advance_float) / 2.0;
		rings.translate(&Point::new(dx, 0.0));

		let mut glyph = if let Some(g) = self.prepare_glyph(&rings) {
			g
		} else {
			return Some(PbfGlyph::empty(index, advance));
		};

		// Render the SDF
		match self.mode {
			RendererMode::Precise => renderer_precise(&mut glyph, rings, self.sdf_step),
			RendererMode::Dummy => renderer_dummy(&mut glyph),
		}

		// Shift the SDF output to re-base the glyph
		glyph.y1 -= GLYPH_SIZE;

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

	/// FNV-1a hash of the metrics and bitmaps of all Fira Sans glyphs in U+0000–U+024F.
	fn hash_rendered_glyphs(renderer: &Renderer) -> u64 {
		let face = Face::parse(TEST_FONT, 0).unwrap();
		let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
		let mut feed = |bytes: &[u8]| {
			for &b in bytes {
				hash ^= b as u64;
				hash = hash.wrapping_mul(0x0100_0000_01b3);
			}
		};
		for index in 0..0x250 {
			if let Some(g) = renderer.render_glyph(&face, index) {
				feed(&g.id.to_le_bytes());
				feed(&g.width.to_le_bytes());
				feed(&g.height.to_le_bytes());
				feed(&g.left.to_le_bytes());
				feed(&g.top.to_le_bytes());
				feed(&g.advance.to_le_bytes());
				feed(g.bitmap.as_deref().unwrap_or_default());
			}
		}
		hash
	}

	#[test]
	fn test_precise_output_is_unchanged() {
		// Golden value: any change to the precise renderer's output changes this hash.
		assert_eq!(
			format!("{:016x}", hash_rendered_glyphs(&Renderer::new_precise())),
			"3493bd77e2ea540c"
		);
	}

	#[test]
	fn test_sdf_step_one_is_unchanged() {
		let renderer = Renderer::new_precise().with_sdf_step(1).unwrap();
		assert_eq!(
			format!("{:016x}", hash_rendered_glyphs(&renderer)),
			"3493bd77e2ea540c"
		);
	}

	#[test]
	fn test_with_sdf_step_validation() {
		for step in [1, 2, 4, 8, 16, 32, 64] {
			assert!(
				Renderer::new_precise().with_sdf_step(step).is_ok(),
				"{step}"
			);
		}
		for step in [0, 3, 6, 100, 128, 255] {
			let err = Renderer::new_precise().with_sdf_step(step).unwrap_err();
			assert!(err.to_string().contains("power of two"), "{step}");
		}
	}

	#[test]
	fn test_sdf_step_quantizes_bitmaps_only() {
		let face = Face::parse(TEST_FONT, 0).unwrap();
		let render_all = |renderer: &Renderer| {
			(0x20..0x180)
				.filter_map(|index| renderer.render_glyph(&face, index))
				.collect::<Vec<_>>()
		};
		let exact = render_all(&Renderer::new_precise());

		for step in [2u8, 4, 8, 16, 32, 64] {
			let quantized = render_all(&Renderer::new_precise().with_sdf_step(step).unwrap());
			assert_eq!(exact.len(), quantized.len());
			let mut changed = false;

			for (a, b) in exact.iter().zip(&quantized) {
				// Metrics are unaffected.
				assert_eq!(
					(a.id, a.width, a.height, a.left, a.top, a.advance),
					(b.id, b.width, b.height, b.left, b.top, b.advance)
				);

				let (Some(bitmap_a), Some(bitmap_b)) = (&a.bitmap, &b.bitmap) else {
					assert_eq!(a.bitmap, b.bitmap);
					continue;
				};
				assert_eq!(bitmap_a.len(), bitmap_b.len());

				for (&va, &vb) in bitmap_a.iter().zip(bitmap_b) {
					// Every value is a multiple of the step, or 255.
					assert!(vb % step == 0 || vb == 255, "step {step}: value {vb}");
					// … and at most half a step away from the exact value.
					assert!(
						(va as i32 - vb as i32).abs() <= step as i32 / 2,
						"step {step}: {va} -> {vb}"
					);
					changed |= va != vb;
				}
			}
			assert!(changed, "step {step} should change some values");
		}
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
		assert_eq!(glyph.advance, 13);
		assert_eq!(
			as_art(&glyph),
			[
				"            ░░░░░░░░░░░░░░░░            ",
				"          ░░░░▒▒▒▒▒▒▒▒▒▒░░░░░░          ",
				"        ░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░          ",
				"        ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒░░░░        ",
				"        ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒░░░░        ",
				"      ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░        ",
				"      ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░      ",
				"      ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░      ",
				"      ░░▒▒▒▒▓▓▓▓▓▓▒▒▓▓▓▓▓▓▒▒▒▒░░░░      ",
				"    ░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░    ",
				"    ░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░    ",
				"    ░░░░▒▒▓▓▓▓▓▓▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░    ",
				"  ░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒░░░░    ",
				"  ░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░  ",
				"  ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░  ",
				"░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░  ",
				"░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░",
				"░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░",
				"░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░",
				"░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░░░░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░",
				"░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░",
				"░░▒▒▒▒▒▒▒▒▒▒▒▒░░░░  ░░░░░░▒▒▒▒▒▒▒▒▒▒░░░░",
				"░░░░░░░░░░░░░░░░░░    ░░░░░░░░░░░░░░░░░░"
			]
		);
	}

	#[test]
	fn test_render_glyph_230() {
		let glyph = get_glyph(230);

		assert_eq!(glyph.width, 19);
		assert_eq!(glyph.height, 14);
		assert_eq!(glyph.left, 0);
		assert_eq!(glyph.top, -11);
		assert_eq!(glyph.advance, 19);
		assert_eq!(
			as_art(&glyph),
			[
				"      ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░      ",
				"    ░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░▒▒▒▒▒▒▒▒▒▒▒▒░░░░░░    ",
				"  ░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░  ",
				"  ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░░░",
				"  ░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░░░",
				"  ░░░░▒▒▒▒▓▓▓▓▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒░░",
				"  ░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░",
				"  ░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒░░",
				"  ░░░░▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░",
				"░░░░▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░",
				"░░░░▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░",
				"░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░",
				"░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░",
				"░░░░▒▒▒▒▓▓▓▓▓▓▒▒▒▒▒▒▒▒▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒▒▒▓▓▒▒▒▒░░░░",
				"░░░░▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░",
				"░░░░▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒░░",
				"  ░░░░▒▒▒▒▒▒▒▒▓▓▓▓▒▒▒▒▒▒▒▒▒▒▒▒▒▒▓▓▓▓▒▒▒▒▒▒▒▒▒▒░░░░",
				"    ░░░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒▒░░░░░░  ",
				"      ░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░░    ",
				"        ░░░░░░░░░░░░░░░░  ░░░░░░░░░░░░░░░░        "
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
				"░░░░▒▒▓▓▓▓▓▓▓▓▓▓▒▒▒▒▒▒▒▒░░",
				"░░░░▒▒▒▒▒▒▓▓▓▓▓▓▓▓▓▓▒▒▒▒░░",
				"░░░░░░▒▒▒▒▒▒▒▒▒▒▓▓▒▒▒▒▒▒░░",
				"  ░░░░░░░░▒▒▒▒▒▒▒▒▒▒▒▒░░░░",
				"      ░░░░░░░░▒▒▒▒▒▒░░░░░░",
				"          ░░░░░░░░░░░░░░  "
			]
		);
	}
}
