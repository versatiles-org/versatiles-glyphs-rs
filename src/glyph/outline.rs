use super::{super::geometry::Rings, ring_builder::RingBuilder};
use ttf_parser::Face;

pub fn build_glyph_outline(code_point: char, face: &Face) -> Option<Rings> {
	let glyph_id = face.glyph_index(code_point)?;
	let mut builder = RingBuilder::new();
	face.outline_glyph(glyph_id, &mut builder);
	Some(builder.into_rings())
}
