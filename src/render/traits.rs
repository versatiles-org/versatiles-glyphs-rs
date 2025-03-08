use super::{RenderResult, BUFFER};
use crate::geometry::{Point, Rings};

/// A trait for rendering glyph outlines into a bitmap representation
/// (such as a signed distance field). The renderer must be [`Copy`],
/// [`Sync`], and [`Send`] to allow parallel operation or safe usage
/// in multiple threads.
pub trait RendererTrait
where
	Self: Copy + Sync + Send,
{
	/// Renders the given `rings` (representing glyph outlines) into a
	/// [`RenderResult`] that contains the bitmap and dimensions.
	///
	/// Implementors typically call [`Self::prepare`] internally
	/// to compute bounding boxes and initialize the bitmap space.
	fn render(&self, rings: Rings) -> Option<RenderResult>;

	/// Prepares the geometry and compute bounding box data for rendering.
	///
	/// This method:
	/// - Computes the bounding box for the given `rings`.
	/// - Adjusts it by adding a `BUFFER` on all sides.
	/// - Translates the outline to ensure it starts at `(0, 0)`.
	/// - Produces a [`RenderResult`] with the computed width, height,
	///   and coordinate offsets.
	///
	/// Returns [`None`] if the bounding box is empty (e.g., no outline data).
	fn prepare(mut rings: Rings) -> Option<(Rings, RenderResult)> {
		// Calculate the real glyph bbox.
		let bbox = rings.get_bbox();

		if bbox.is_empty() {
			return None;
		}

		let x0 = bbox.min.x.floor() as i32 - BUFFER;
		let y0 = bbox.min.y.floor() as i32 - BUFFER;
		let x1 = bbox.max.x.ceil() as i32 + BUFFER;
		let y1 = bbox.max.y.ceil() as i32 + BUFFER;
		let width = (x1 - x0) as usize;
		let height = (y1 - y0) as usize;

		// Offset so that glyph outlines are in the bounding box.
		let offset = Point::from((-x0, -y0));
		rings.translate(&offset);

		let glyph = RenderResult {
			x0,
			y1,
			x1,
			y0,
			width: width as u32,
			height: height as u32,
			bitmap: None,
		};

		Some((rings, glyph))
	}
}
