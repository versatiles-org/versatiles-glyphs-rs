use super::{
	rtree_segments::{min_distance_to_line_segment, SegmentValue},
	RendererTrait, SdfGlyph, CUTOFF,
};
use crate::geometry::{Point, Rings};
use rstar::RTree;

#[derive(Clone, Copy)]
pub struct RendererPrecise {}

impl RendererTrait for RendererPrecise {
	// https://github.com/mapbox/sdf-glyph-foundry/blob/6ed4f2099009fc8a1a324626345ceb29dcd5277c/include/mapbox/glyph_foundry_impl.hpp
	fn render(&self, rings: Rings) -> Option<SdfGlyph> {
		let (rings, mut glyph) = Self::prepare(rings)?;

		let width = glyph.width as usize;
		let height = glyph.height as usize;

		// Build a R-tree of line segments
		let segments = rings
			.get_segments()
			.into_iter()
			.map(SegmentValue::new)
			.collect::<Vec<SegmentValue>>();

		let rtree = RTree::bulk_load(segments);

		// 5) For each pixel, compute distance, check inside-ness, etc.

		let mut bitmap = vec![0; width * height];

		let max_radius = 8.0;
		let radius_by_256 = 256.0 / max_radius;

		for y in 0..height {
			for x in 0..width {
				// We'll invert Y to match typical image coordinate systems
				let i = (height - 1 - y) * width + x;

				// The sample point is the center of the pixel
				let sample_pt = Point::new((x as f64) + 0.5, (y as f64) + 0.5);

				// Distance from the outline
				let mut d = min_distance_to_line_segment(&rtree, &sample_pt, &max_radius);

				// If the point is inside, invert
				if rings.contains_point(&sample_pt) {
					d = -d;
				}

				d = d * radius_by_256 + CUTOFF;

				let n = (255.0 - d).clamp(0.0, 255.0);

				// The final code does 255 - n, so that "outside" is white,
				// "inside" is black, or vice versa:
				bitmap[i] = n.round() as u8;
			}
		}

		glyph.bitmap = Some(bitmap);

		Some(glyph)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{geometry::Rings, utils::bitmap_as_digit_art};

	fn make_square_rings() -> Rings {
		Rings::from(vec![vec![(1, 2), (5, 2), (5, 6), (1, 6), (1, 2)]])
	}

	fn make_empty_rings() -> Rings {
		Rings::default()
	}

	#[test]
	fn test_render_sdf_empty_bbox() {
		let rings = make_empty_rings();
		let glyph = RendererPrecise {}.render(rings);
		assert!(glyph.is_none(), "Expected None for empty geometry");
	}

	#[test]
	fn test_render_sdf_simple_square() {
		let rings = make_square_rings();
		let glyph = RendererPrecise {}.render(rings).unwrap();

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
