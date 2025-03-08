use super::{RenderResult, RendererTrait};
use crate::geometry::Rings;

/// A simple renderer that produces a blank bitmap of the appropriate size
/// but does not perform any actual rendering of the glyph outline.
///
/// This is primarily useful as a placeholder or for testing scenarios
/// where only the dimensions/metrics matter rather than the rendered output.
#[derive(Clone, Copy)]
pub struct RendererDummy {}

impl RendererTrait for RendererDummy {
	fn render(&self, rings: Rings) -> Option<RenderResult> {
		// Prepare a default RenderResult object from the geometry,
		// then fill its bitmap with zeroes (representing an empty / unpainted glyph).
		let (_rings, mut glyph) = Self::prepare(rings)?;
		glyph.bitmap = Some(vec![0; (glyph.width * glyph.height) as usize]);
		Some(glyph)
	}
}
