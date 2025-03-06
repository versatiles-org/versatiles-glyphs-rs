use super::{RendererTrait, SdfGlyph};
use crate::geometry::Rings;

#[derive(Clone, Copy)]
pub struct RendererDummy {}

impl RendererTrait for RendererDummy {
	fn render(&self, rings: Rings) -> Option<SdfGlyph> {
		let (_rings, mut glyph) = Self::prepare(rings)?;
		glyph.bitmap = Some(vec![0; (glyph.width * glyph.height) as usize]);
		Some(glyph)
	}
}
