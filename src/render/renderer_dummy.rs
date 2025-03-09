use super::RenderResult;

pub fn renderer_dummy(glyph: &mut RenderResult) {
	glyph.bitmap = Some(vec![0; (glyph.width * glyph.height) as usize]);
}
