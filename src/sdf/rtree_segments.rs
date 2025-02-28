use super::super::geometry::{Point, Segment};
use rstar::{RTree, RTreeObject, AABB};

#[derive(Clone)]
pub struct SegmentValue {
	segment: Segment,
}

impl SegmentValue {
	pub fn new(segment: Segment) -> Self {
		SegmentValue { segment }
	}
}

impl RTreeObject for SegmentValue {
	type Envelope = AABB<[f32; 2]>;

	fn envelope(&self) -> Self::Envelope {
		let minx = self.segment.start.x.min(self.segment.end.x);
		let maxx = self.segment.start.x.max(self.segment.end.x);
		let miny = self.segment.start.y.min(self.segment.end.y);
		let maxy = self.segment.start.y.max(self.segment.end.y);
		AABB::from_corners([minx, miny], [maxx, maxy])
	}
}

pub fn min_distance_to_line_segment(rtree: &RTree<SegmentValue>, p: Point, radius: f32) -> f32 {
	// We'll do a bounding box query
	let query_env = AABB::from_corners([p.x - radius, p.y - radius], [p.x + radius, p.y + radius]);
	let candidates = rtree.locate_in_envelope_intersecting(&query_env);

	let mut best_sq = f32::INFINITY;
	let squared_rad = radius * radius;
	for candidate in candidates {
		let seg = candidate.segment;
		let dist_sq = seg.squared_distance_to_point(&p);
		if dist_sq < best_sq && dist_sq < squared_rad {
			best_sq = dist_sq;
		}
	}

	best_sq.sqrt()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_envelope_computation() {
		let seg = Segment {
			start: Point { x: 1.0, y: 4.0 },
			end: Point { x: 3.0, y: 2.0 },
		};
		let value = SegmentValue::new(seg);

		let envelope = value.envelope();
		let min_corner = envelope.lower();
		let max_corner = envelope.upper();

		assert_eq!(min_corner, [1.0, 2.0]);
		assert_eq!(max_corner, [3.0, 4.0]);
	}

	#[test]
	fn test_min_distance_single_segment() {
		// Create one segment
		let seg = Segment {
			start: Point { x: 0.0, y: 0.0 },
			end: Point { x: 4.0, y: 0.0 },
		};
		let rtree = RTree::bulk_load(vec![SegmentValue::new(seg)]);

		// Query near the segment
		let p = Point { x: 2.0, y: 1.0 };
		let radius = 5.0;
		let dist = min_distance_to_line_segment(&rtree, p, radius);
		// Distance from (2,1) to segment (0,0)-(4,0) is 1.0
		assert!((dist - 1.0).abs() < f32::EPSILON);
	}

	#[test]
	fn test_min_distance_out_of_range() {
		// Single horizontal segment from (0,0) to (4,0)
		let seg = Segment {
			start: Point { x: 0.0, y: 0.0 },
			end: Point { x: 4.0, y: 0.0 },
		};
		let rtree = RTree::bulk_load(vec![SegmentValue::new(seg)]);

		// The query point is far away, with a small radius
		let p = Point { x: 100.0, y: 100.0 };
		let radius = 5.0; // bounding box won't intersect the segment
		let dist = min_distance_to_line_segment(&rtree, p, radius);

		// The function returns sqrt(INFINITY) if there was no intersection
		// i.e. no candidate within the bounding box => best_sq remains f32::INFINITY
		// sqrt(INFINITY) is still f32::INFINITY.
		assert!(dist.is_infinite());
	}

	#[test]
	fn test_min_distance_multiple_segments() {
		// Create multiple segments
		let seg1 = Segment {
			start: Point { x: 0.0, y: 0.0 },
			end: Point { x: 4.0, y: 0.0 },
		};
		let seg2 = Segment {
			start: Point { x: 2.0, y: 2.0 },
			end: Point { x: 2.0, y: 6.0 },
		};
		let seg3 = Segment {
			start: Point { x: -1.0, y: -1.0 },
			end: Point { x: -1.0, y: -5.0 },
		};

		// Build RTree
		let rtree = RTree::bulk_load(vec![
			SegmentValue::new(seg1),
			SegmentValue::new(seg2),
			SegmentValue::new(seg3),
		]);

		// We'll pick a point near seg1 and seg2
		// Let p = (2,1), a small radius that includes seg1 and seg2 envelopes
		let p = Point { x: 2.0, y: 1.0 };
		let radius = 5.0;
		let dist = min_distance_to_line_segment(&rtree, p, radius);
		// The closest distance:
		// - seg1 is the line from (0,0) to (4,0), distance 1.0
		// - seg2 is the line from (2,2) to (2,6), distance 1.0
		// They are both equally 1.0 away
		assert!((dist - 1.0).abs() < f32::EPSILON);

		// Now pick a point near seg3
		let p2 = Point { x: -1.0, y: -3.0 };
		let dist2 = min_distance_to_line_segment(&rtree, p2, radius);
		// p2 is on seg3's line, so distance is zero
		assert!((dist2 - 0.0).abs() < f32::EPSILON);
	}

	#[test]
	fn test_min_distance_exact_on_segment() {
		// Create a simple segment
		let seg = Segment {
			start: Point { x: 1.0, y: 1.0 },
			end: Point { x: 5.0, y: 1.0 },
		};
		let rtree = RTree::bulk_load(vec![SegmentValue::new(seg)]);

		// Query with a point that lies exactly on the segment
		let p = Point { x: 3.0, y: 1.0 };
		let radius = 2.0;
		let dist = min_distance_to_line_segment(&rtree, p, radius);

		// Distance should be zero
		assert!((dist - 0.0).abs() < f32::EPSILON);
	}
}
