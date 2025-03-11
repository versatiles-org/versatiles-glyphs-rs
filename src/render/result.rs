use super::BUFFER;
use crate::protobuf::PbfGlyph;

/// Holds intermediate results of the glyph rendering process,
/// including bitmap dimensions and offset bounds.
#[allow(dead_code)]
#[derive(Debug, Default)]
pub struct RenderResult {
	/// The minimum x-coordinates of the rendered glyph.
	pub x0: i32,
	/// The maximum x-coordinates of the rendered glyph.
	pub x1: i32,
	/// The minimum y-coordinates of the rendered glyph.
	pub y0: i32,
	/// The maximum y-coordinates of the rendered glyph.
	pub y1: i32,

	/// The width of the rendered bitmap, including any buffer or padding.
	pub width: u32,
	/// The height of the rendered bitmap, including any buffer or padding.
	pub height: u32,

	/// The rendered bitmap data, if available.
	pub bitmap: Option<Vec<u8>>,
}

impl RenderResult {
	/// Consumes this rendering result and produces a [`PbfGlyph`].
	///
	/// It subtracts the buffer from the raw width/height, then
	/// adjusts the `left` and `top` metrics to account for that buffer,
	/// returning a fully initialized `PbfGlyph`.
	///
	/// # Examples
	///
	/// ```
	/// use versatiles_glyphs::render::RenderResult;
	/// use versatiles_glyphs::protobuf::PbfGlyph;
	///
	/// let render = RenderResult {
	///     x0: 0,
	///     x1: 14,
	///     y0: -7,
	///     y1: 10,
	///     width: 20,
	///     height: 24,
	///     bitmap: Some(vec![0; 20 * 24]),
	/// };
	///
	/// let glyph: PbfGlyph = render.into_pbf_glyph(65, 14);
	/// assert_eq!(glyph.id, 65);
	/// assert_eq!(glyph.advance, 14);
	/// ```
	pub fn into_pbf_glyph(self, id: u32, advance: u32) -> PbfGlyph {
		PbfGlyph {
			id,
			bitmap: self.bitmap,
			width: self.width - 2 * BUFFER as u32,
			height: self.height - 2 * BUFFER as u32,
			left: self.x0 + BUFFER,
			top: self.y1 - BUFFER,
			advance,
		}
	}
}
