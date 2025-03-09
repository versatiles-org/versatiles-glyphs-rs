use super::{
	rtree_segments::{min_distance_to_line_segment, SegmentValue},
	RenderResult, CUTOFF,
};
use crate::geometry::{Point, Rings};
use rstar::RTree;

pub fn renderer_precise(glyph: &mut RenderResult, rings: Rings) {
	let width = glyph.width as usize;
	let height = glyph.height as usize;

	// Build an R-tree of outline segments from the ring geometry.
	let segments = rings
		.get_segments()
		.into_iter()
		.map(SegmentValue::new)
		.collect::<Vec<SegmentValue>>();
	let rtree = RTree::bulk_load(segments);

	// Initialize the bitmap. We'll fill each pixel with the SDF value.
	let mut bitmap = vec![0; width * height];

	let max_radius = 8.0;
	let radius_by_256 = 256.0 / max_radius;

	let x0 = glyph.x0 as f64 + 0.5;
	let y0 = glyph.y0 as f64 + 0.5;

	// For each pixel in the bounding box, compute signed distance
	// from the outline, then clamp the result to [0..255].
	for y in 0..height {
		for x in 0..width {
			let sample_pt = Point::new(x as f64 + x0, y as f64 + y0);

			// Distance from the outline.
			let mut d = min_distance_to_line_segment(&rtree, &sample_pt, &max_radius);

			// Invert the distance if we're inside the outline.
			if rings.contains_point(&sample_pt) {
				d = -d;
			}

			// Scale distance and apply the cutoff for the final SDF value.
			d = d * radius_by_256 + CUTOFF;
			let n = (255.0 - d).clamp(0.0, 255.0);

			let i = (height - 1 - y) * width + x; // Invert Y axis
			bitmap[i] = n.round() as u8;
		}
	}

	glyph.bitmap = Some(bitmap);
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{geometry::Rings, utils::bitmap_as_digit_art};

	fn make_square_rings() -> Rings {
		Rings::from(vec![vec![(1, 2), (5, 2), (5, 6), (1, 6), (1, 2)]])
	}

	#[test]
	fn test_render_sdf_simple_square() {
		let rings = make_square_rings();
		let mut glyph = RenderResult {
			width: 10,
			height: 10,
			x0: -2,
			x1: 8,
			y0: -1,
			y1: 9,
			bitmap: None,
		};
		renderer_precise(&mut glyph, rings);

		assert_eq!(glyph.width, 10);
		assert_eq!(glyph.height, 10);
		assert_eq!(glyph.x0, -2);
		assert_eq!(glyph.x1, 8);
		assert_eq!(glyph.y0, -1);
		assert_eq!(glyph.y1, 9);
		assert_eq!(
			glyph.bitmap.as_ref().unwrap().len(),
			(glyph.width * glyph.height) as usize
		);

		assert_eq!(
			bitmap_as_digit_art(&glyph.bitmap.unwrap(), glyph.width as usize),
			vec![
				"30 38 42 43 43 43 43 42 38 30",
				"38 48 54 55 55 55 55 54 48 38",
				"42 54 65 68 68 68 68 65 54 42",
				"43 55 68 80 80 80 80 68 55 43",
				"43 55 68 80 93 93 80 68 55 43",
				"43 55 68 80 93 93 80 68 55 43",
				"43 55 68 80 80 80 80 68 55 43",
				"42 54 65 68 68 68 68 65 54 42",
				"38 48 54 55 55 55 55 54 48 38",
				"30 38 42 43 43 43 43 42 38 30"
			]
		);
	}
}
