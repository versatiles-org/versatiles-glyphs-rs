use super::{super::geometry::Rings, ring_builder::RingBuilder};
use ttf_parser::Face;

pub fn build_glyph_outline(code_point: char, face: &Face, size: f32) -> Option<Rings> {
	let glyph_id = face.glyph_index(code_point)?;
	let mut builder = RingBuilder::new();
	face.outline_glyph(glyph_id, &mut builder);

	let mut rings = builder.into_rings();
	let scale = size / face.height() as f32;
	rings.scale(scale);

	return Some(rings);
}
