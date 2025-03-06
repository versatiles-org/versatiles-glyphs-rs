use super::{SdfGlyph, BUFFER};
use crate::geometry::{Point, Rings};

pub trait RendererTrait
where
	Self: Copy + Sync + Send,
{
	fn render(&self, rings: Rings) -> Option<SdfGlyph>;

	fn prepare(mut rings: Rings) -> Option<(Rings, SdfGlyph)> {
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

		let glyph = SdfGlyph {
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
