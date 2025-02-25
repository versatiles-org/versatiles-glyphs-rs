use std::f32::INFINITY;
use rstar::{RTree, RTreeObject, AABB};
use super::super::geometry::{Point, Segment};

#[derive(Clone)]
pub struct SegmentValue {
	segment: Segment,
}

impl SegmentValue {
	pub fn new(segment: Segment) -> Self {
		SegmentValue { segment }
	}
}

// Implement RTreeObject
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

	let mut best_sq = INFINITY;
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
