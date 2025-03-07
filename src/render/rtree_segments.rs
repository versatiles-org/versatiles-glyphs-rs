use super::super::geometry::{Point, Segment};
use rstar::{RTree, RTreeObject, AABB};

/// A wrapper for a [`Segment`], allowing it to be inserted into an [`rstar::RTree`].
///
/// This enables fast spatial queries, such as finding the nearest segment
/// to a particular point when building a signed distance field.
#[derive(Clone, Debug)]
pub struct SegmentValue<'a> {
	segment: Segment<'a>,
}

impl<'a> SegmentValue<'a> {
	/// Creates a new [`SegmentValue`] from a [`Segment`].
	pub fn new(segment: Segment<'a>) -> Self {
		SegmentValue { segment }
	}
}

impl RTreeObject for SegmentValue<'_> {
	type Envelope = AABB<[f64; 2]>;

	/// Returns the axis-aligned bounding box (AABB) covering the segment,
	/// which is used by the R-tree for indexing.
	fn envelope(&self) -> Self::Envelope {
		let minx = self.segment.start.x.min(self.segment.end.x);
		let maxx = self.segment.start.x.max(self.segment.end.x);
		let miny = self.segment.start.y.min(self.segment.end.y);
		let maxy = self.segment.start.y.max(self.segment.end.y);
		AABB::from_corners([minx, miny], [maxx, maxy])
	}
}

/// Finds the shortest distance from a point `p` to any line segment in an [`RTree`],
/// searching only segments intersecting a bounding box defined by `max_radius`.
///
/// This function helps optimize distance computations for glyph outline rendering:
/// rather than searching every segment, we only look at those near the query point.
#[inline]
pub fn min_distance_to_line_segment(
	rtree: &RTree<SegmentValue>,
	p: &Point,
	max_radius: &f64,
) -> f64 {
	// Create a bounding box centered on `p` with side length = 2 * max_radius.
	// This quickly filters out segments outside the maximum distance of interest.
	let query_env = AABB::from_corners(
		[p.x - max_radius, p.y - max_radius],
		[p.x + max_radius, p.y + max_radius],
	);

	// Collect candidate segments in that bounding box.
	let candidates = rtree.locate_in_envelope_intersecting(&query_env);

	// Determine the squared distance from `p` to each candidate,
	// keeping track of the minimum.
	let mut best_sq = f64::INFINITY;
	for candidate in candidates {
		let seg = &candidate.segment;
		let dist_sq = seg.squared_distance_to_point(p);
		if dist_sq < best_sq {
			best_sq = dist_sq;
		}
	}

	// Return the actual distance (not squared).
	best_sq.sqrt()
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_envelope_computation() {
		let start = Point { x: 1.0, y: 4.0 };
		let end = Point { x: 3.0, y: 2.0 };
		let seg = Segment {
			start: &start,
			end: &end,
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
		let start = Point { x: 0.0, y: 0.0 };
		let end = Point { x: 4.0, y: 0.0 };
		let seg = Segment {
			start: &start,
			end: &end,
		};
		let rtree = RTree::bulk_load(vec![SegmentValue::new(seg)]);

		// Query near the segment
		let p = Point { x: 2.0, y: 1.0 };
		let radius = 5.0;
		let dist = min_distance_to_line_segment(&rtree, &p, &radius);
		// Distance from (2,1) to segment (0,0)-(4,0) is 1.0
		assert!((dist - 1.0).abs() < f64::EPSILON);
	}

	#[test]
	fn test_min_distance_out_of_range() {
		// Single horizontal segment from (0,0) to (4,0)
		let start = Point { x: 0.0, y: 0.0 };
		let end = Point { x: 4.0, y: 0.0 };
		let seg = Segment {
			start: &start,
			end: &end,
		};
		let rtree = RTree::bulk_load(vec![SegmentValue::new(seg)]);

		// The query point is far away, with a small radius
		let p = Point { x: 100.0, y: 100.0 };
		let radius = 5.0; // bounding box won't intersect the segment
		let dist = min_distance_to_line_segment(&rtree, &p, &radius);

		// The function returns sqrt(INFINITY) if there was no intersection
		// i.e. no candidate within the bounding box => best_sq remains f64::INFINITY
		// sqrt(INFINITY) is still f64::INFINITY.
		assert!(dist.is_infinite());
	}

	#[test]
	fn test_min_distance_multiple_segments() {
		// Create multiple segments
		let start = Point { x: 0.0, y: 0.0 };
		let end = Point { x: 4.0, y: 0.0 };
		let seg1 = Segment {
			start: &start,
			end: &end,
		};
		let start = Point { x: 2.0, y: 2.0 };
		let end = Point { x: 2.0, y: 6.0 };
		let seg2 = Segment {
			start: &start,
			end: &end,
		};
		let start = Point { x: -1.0, y: -1.0 };
		let end = Point { x: -1.0, y: -5.0 };
		let seg3 = Segment {
			start: &start,
			end: &end,
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
		let dist = min_distance_to_line_segment(&rtree, &p, &radius);
		// The closest distance:
		// - seg1 is the line from (0,0) to (4,0), distance 1.0
		// - seg2 is the line from (2,2) to (2,6), distance 1.0
		// They are both equally 1.0 away
		assert!((dist - 1.0).abs() < f64::EPSILON);

		// Now pick a point near seg3
		let p2 = Point { x: -1.0, y: -3.0 };
		let dist2 = min_distance_to_line_segment(&rtree, &p2, &radius);
		// p2 is on seg3's line, so distance is zero
		assert!((dist2 - 0.0).abs() < f64::EPSILON);
	}

	#[test]
	fn test_min_distance_exact_on_segment() {
		// Create a simple segment
		let start = Point { x: 1.0, y: 1.0 };
		let end = Point { x: 5.0, y: 1.0 };
		let seg = Segment {
			start: &start,
			end: &end,
		};
		let rtree = RTree::bulk_load(vec![SegmentValue::new(seg)]);

		// Query with a point that lies exactly on the segment
		let p = Point { x: 3.0, y: 1.0 };
		let radius = 2.0;
		let dist = min_distance_to_line_segment(&rtree, &p, &radius);

		// Distance should be zero
		assert!((dist - 0.0).abs() < f64::EPSILON);
	}
}
