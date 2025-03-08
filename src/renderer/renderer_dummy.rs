use super::{RenderResult, RendererTrait};
use crate::geometry::Rings;

#[derive(Clone, Copy)]
pub struct RendererDummy {}

impl RendererTrait for RendererDummy {
	fn render(&self, rings: Rings) -> Option<RenderResult> {
		let (_rings, mut glyph) = Self::prepare(rings)?;
		glyph.bitmap = Some(vec![0; (glyph.width * glyph.height) as usize]);
		Some(glyph)
	}
}
