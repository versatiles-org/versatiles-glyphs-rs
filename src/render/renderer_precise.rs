use super::{
	rtree_segments::{min_distance_to_line_segment, SegmentValue},
	RenderResult, CUTOFF,
};
use crate::geometry::{Point, Rings};
use rstar::RTree;

pub fn renderer_precise(glyph: &mut RenderResult, rings: Rings) {
	let width = glyph.width as usize;
	let height = glyph.height as usize;

	// Pull the segments once; reuse for both the R-tree (distance lookup) and
	// the per-row scanline winding (inside/outside lookup).
	let segments = rings.get_segments();
	let rtree = RTree::bulk_load(
		segments
			.iter()
			.cloned()
			.map(SegmentValue::new)
			.collect::<Vec<SegmentValue>>(),
	);

	let mut bitmap = vec![0; width * height];

	let max_radius = 8.0;
	let radius_by_256 = 256.0 / max_radius;

	let x0 = glyph.x0 as f64 + 0.5;
	let y0 = glyph.y0 as f64 + 0.5;

	// Reused scratch space for crossings on the current row.
	let mut crossings: Vec<(f64, i32)> = Vec::new();

	for y in 0..height {
		let py = y as f64 + y0;

		// Collect signed x-crossings of the horizontal ray at y = py against
		// every segment. Conventions match `Ring::winding_number`:
		//   upward   crossing (s.y <= py < e.y) → +1
		//   downward crossing (s.y >  py >= e.y) → -1
		crossings.clear();
		for seg in &segments {
			let s = seg.start;
			let e = seg.end;
			if s.y <= py && e.y > py {
				let t = (py - s.y) / (e.y - s.y);
				crossings.push((s.x + t * (e.x - s.x), 1));
			} else if s.y > py && e.y <= py {
				let t = (py - s.y) / (e.y - s.y);
				crossings.push((s.x + t * (e.x - s.x), -1));
			}
		}
		crossings.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

		// Closed rings produce equal up/down crossings per row, so the initial
		// winding number (counting all crossings as "to the right of px=-∞")
		// sums to 0. As `px` sweeps right, each crossing we pass moves from
		// "to the right" to "to the left", so we subtract its sign.
		let mut wn: i32 = 0;
		let mut idx = 0usize;

		for x in 0..width {
			let px = x as f64 + x0;
			while idx < crossings.len() && crossings[idx].0 <= px {
				wn -= crossings[idx].1;
				idx += 1;
			}
			let inside = wn != 0;

			let sample_pt = Point::new(px, py);
			let mut d = min_distance_to_line_segment(&rtree, &sample_pt, &max_radius);
			if inside {
				d = -d;
			}

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
