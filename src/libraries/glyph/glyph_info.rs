
#[derive(Debug, Default)]
pub struct GlyphInfo {
	pub advance: f64,

	pub left: i32,
	pub top: i32,

	pub width: u32,
	pub height: u32,

	pub bitmap: Vec<u8>,
}
