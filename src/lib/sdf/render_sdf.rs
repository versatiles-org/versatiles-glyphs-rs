use super::{
	super::geometry::Point,
	rtree_segments::{min_distance_to_line_segment, SegmentValue},
};
use crate::geometry::Rings;
use rstar::RTree;

#[derive(Debug, Default)]
pub struct SdfGlyph {
	pub left: i32,
	pub top: i32,

	pub width: u32,
	pub height: u32,

	pub bitmap: Vec<u8>,
}

// https://github.com/mapbox/sdf-glyph-foundry/blob/6ed4f2099009fc8a1a324626345ceb29dcd5277c/include/mapbox/glyph_foundry_impl.hpp
pub fn render_sdf(mut rings: Rings, buffer: usize, cutoff: f32) -> Option<SdfGlyph> {
	// Calculate the real glyph bbox.
	let bbox = rings.get_bbox();

	if bbox.is_empty() {
		return None;
	}

	// Offset so that glyph outlines are in the bounding box.
	let offset = bbox
		.min
		.clone()
		.inverted()
		.translated(Point::new(buffer as f32, buffer as f32));

	rings.translate(offset);

	// Build a R-tree of line segments
	let segments = rings
		.get_segments()
		.into_iter()
		.map(SegmentValue::new)
		.collect::<Vec<SegmentValue>>();

	let rtree = RTree::bulk_load(segments);

	// 5) For each pixel, compute distance, check inside-ness, etc.
	let width = bbox.width() as usize + 2 * buffer;
	let height = bbox.height() as usize + 2 * buffer;

	let mut bitmap = Vec::new();
	bitmap.resize(width * height, 0);

	let offset = 0.5f32;
	let radius = 8.0;
	let radius_by_256 = 256.0 / radius;

	for y in 0..height {
		for x in 0..width {
			// We'll invert Y to match typical image coordinate systems
			let i = (height - 1 - y) * (width) + x;

			// The sample point is the center of the pixel
			let sample_pt = Point::new((x as f32) + offset, (y as f32) + offset);

			// Distance from the outline
			let mut d = min_distance_to_line_segment(&rtree, sample_pt, radius) * radius_by_256;

			// If the point is inside, invert
			if rings.contains_point(&sample_pt) {
				d = -d;
			}

			d += cutoff * 256.0;

			let mut n = d.round() as i32;
			if n < 0 {
				n = 0;
			} else if n > 255 {
				n = 255;
			}

			// The final code does 255 - n, so that "outside" is white,
			// "inside" is black, or vice versa:
			bitmap[i] = (255 - n as u8) as u8;
		}
	}

	Some(SdfGlyph {
		left: bbox.min.x as i32,
		top: bbox.min.y as i32,
		width: width as u32,
		height: height as u32,
		bitmap,
	})
}
