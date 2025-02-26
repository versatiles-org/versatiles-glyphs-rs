use super::{
	super::{
		geometry::Point,
		glyph::{build_glyph_outline, GlyphInfo},
	},
	rtree::{min_distance_to_line_segment, SegmentValue},
};
use rstar::RTree;
use ttf_parser::Face;

// https://github.com/mapbox/sdf-glyph-foundry/blob/6ed4f2099009fc8a1a324626345ceb29dcd5277c/include/mapbox/glyph_foundry_impl.hpp
pub fn render_sdf(
	code_point: char,
	face: &Face,
	buffer: i32,
	cutoff: f32,
	size: u32,
) -> Option<GlyphInfo> {
	let glyph_id = face.glyph_index(code_point)?;

	let mut rings = build_glyph_outline(glyph_id, face);

	let scale = size as f32 / face.height() as f32;
	rings.scale(scale);

	// Calculate the real glyph bbox.
	let bbox = rings.get_bbox();

	if bbox.width() == 0.0 || bbox.height() == 0.0 {
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
	let buffered_width = bbox.width() as usize + 2 * buffer as usize;
	let buffered_height = bbox.height() as usize + 2 * buffer as usize;

	let mut bitmap = Vec::new();
	bitmap.resize(buffered_width * buffered_height, 0);

	let offset = 0.5f32;
	let radius = 8.0;
	let radius_by_256 = 256.0 / radius;

	for y in 0..buffered_height {
		for x in 0..buffered_width {
			// We'll invert Y to match typical image coordinate systems
			let i = (buffered_height - 1 - y) * (buffered_width) + x;

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

	Some(GlyphInfo {
		advance: face.glyph_hor_advance(glyph_id).unwrap() as f64 / 64.0,
		//ascender: face.ascender() as f64 / 64.0,
		//descender: face.descender() as f64 / 64.0,
		//line_height: (face.height() + face.line_gap()) as f64 / 64.0,
		//code_point,
		left: bbox.min.x as i32,
		top: bbox.min.y as i32,
		width: bbox.width().ceil() as u32,
		height: bbox.height().ceil() as u32,
		bitmap,
	})
}
