use super::super::geometry::Rings;
use super::RingBuilder;
use ttf_parser::{Face, GlyphId};

pub fn build_glyph_outline(glyph_id: GlyphId, face: &Face) -> Rings {
	let mut builder = RingBuilder::new();
	face.outline_glyph(glyph_id, &mut builder);
	builder.into_rings()
}
