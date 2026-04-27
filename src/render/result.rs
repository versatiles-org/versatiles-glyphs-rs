use super::BUFFER;
use crate::protobuf::PbfGlyph;

/// Holds intermediate results of the glyph rendering process,
/// including bitmap dimensions and offset bounds.
#[derive(Debug, Default)]
pub struct RenderResult {
	/// The minimum x-coordinates of the rendered glyph.
	pub x0: i32,
	/// The maximum x-coordinates of the rendered glyph.
	///
	/// Populated by the renderer for completeness. Internal consumers derive
	/// the right edge from `x0 + width` instead, so this field is unread
	/// by the rendering pipeline; it remains as part of the public struct.
	#[allow(dead_code)]
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
	/// The bitmap stored on disk is `(width + 2·BUFFER) × (height + 2·BUFFER)`
	/// pixels: a content area surrounded by `BUFFER` pixels of SDF
	/// padding on every side. The PBF metrics report only the *content area*
	/// (`width`, `height`, `left`, `top`) — consumers reconstruct the full
	/// bitmap dimensions by adding back `2·BUFFER` on each axis.
	///
	/// `left = x0 + BUFFER` and `top = y1 - BUFFER` therefore correspond to
	/// `floor(min.x)` and `ceil(max.y)` of the float bbox computed in
	/// [`Renderer::prepare_glyph`](crate::render::Renderer). See the
	/// [`render` module docs](crate::render) for why those `floor`/`ceil`
	/// boundaries don't always coincide with the actual outline.
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
