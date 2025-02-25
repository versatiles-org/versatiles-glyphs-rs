#![allow(dead_code)]

use super::{BBox, Point, Ring, Segment};

pub struct Rings {
	pub rings: Vec<Ring>,
}

impl Rings {
	pub fn new() -> Self {
		Rings { rings: Vec::new() }
	}

	pub fn len(&self) -> usize {
		self.rings.len()
	}

	pub fn add_ring(&mut self, ring: Ring) {
		self.rings.push(ring);
	}

	pub fn get_bbox(&self) -> BBox {
		let mut bbox = BBox::new();
		for ring in &self.rings {
			bbox.include_bbox(&ring.get_bbox());
		}
		bbox
	}

	pub fn translate(&mut self, offset: Point) {
		for ring in &mut self.rings {
			ring.translate(offset);
		}
	}

	pub fn get_segments(&self) -> Vec<Segment> {
		self
			.rings
			.iter()
			.flat_map(|ring| ring.get_segments())
			.collect()
	}

	pub fn contains_point(&self, pt: &Point) -> bool {
		let mut winding_number = 0;
		for ring in self.rings.iter() {
			winding_number += ring.winding_number(pt);
		}
		winding_number != 0
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_rings_new() {
		let rings = Rings::new();
		assert_eq!(rings.rings.len(), 0);
	}

	#[test]
	fn test_add_ring() {
		let mut rings = Rings::new();
		let ring = Ring::new();
		rings.add_ring(ring);
		assert_eq!(rings.rings.len(), 1);
	}

	#[test]
	fn test_get_bbox_multiple_rings() {
		let mut rings = Rings::new();
		// Ring 1: bounding box from (0,0) to (2,2)
		let mut ring1 = Ring::new();
		ring1.add_point(Point::new(0.0, 0.0));
		ring1.add_point(Point::new(2.0, 2.0));
		ring1.close();

		// Ring 2: bounding box from (3, -1) to (5, 1)
		let mut ring2 = Ring::new();
		ring2.add_point(Point::new(3.0, -1.0));
		ring2.add_point(Point::new(5.0, 1.0));
		ring2.close();

		rings.add_ring(ring1);
		rings.add_ring(ring2);

		let bbox = rings.get_bbox();
		// The combined bounding box should be from min(0, -1) => (0, -1)
		// to max(5, 2) => (5, 2)
		assert_eq!(bbox.min.x, 0.0);
		assert_eq!(bbox.min.y, -1.0);
		assert_eq!(bbox.max.x, 5.0);
		assert_eq!(bbox.max.y, 2.0);
	}

	#[test]
	fn test_translate() {
		let mut rings = Rings::new();
		let mut ring = Ring::new();
		ring.add_point(Point::new(0.0, 0.0));
		ring.add_point(Point::new(1.0, 1.0));
		ring.close();

		rings.add_ring(ring);
		// Now translate everything by (2, 3)
		rings.translate(Point::new(2.0, 3.0));

		// The ring's points should each be offset
		let translated_ring = &rings.rings[0];
		// Original points: (0,0), (1,1), (0,0) again
		// After translation: (2,3), (3,4), (2,3)
		assert_eq!(translated_ring.points[0].x, 2.0);
		assert_eq!(translated_ring.points[0].y, 3.0);
		assert_eq!(translated_ring.points[1].x, 3.0);
		assert_eq!(translated_ring.points[1].y, 4.0);
		// The last point is the closed ring's first point repeated
		assert_eq!(translated_ring.points[2].x, 2.0);
		assert_eq!(translated_ring.points[2].y, 3.0);
	}

	#[test]
	fn test_get_segments() {
		let mut rings = Rings::new();

		// First ring has 3 points => 2 segments
		let mut ring1 = Ring::new();
		ring1.add_point(Point::new(0.0, 0.0));
		ring1.add_point(Point::new(1.0, 0.0));
		ring1.add_point(Point::new(1.0, 1.0));

		// Second ring has 4 points => 3 segments
		let mut ring2 = Ring::new();
		ring2.add_point(Point::new(2.0, 2.0));
		ring2.add_point(Point::new(3.0, 2.0));
		ring2.add_point(Point::new(3.0, 3.0));
		ring2.add_point(Point::new(2.0, 3.0));

		rings.add_ring(ring1);
		rings.add_ring(ring2);

		let segments = rings.get_segments();
		// total segments = 2 + 3 = 5
		assert_eq!(segments.len(), 5);

		// Just verify a few
		// The first ring's first segment: (0,0)->(1,0)
		assert_eq!(segments[0].start.x, 0.0);
		assert_eq!(segments[0].start.y, 0.0);
		assert_eq!(segments[0].end.x, 1.0);
		assert_eq!(segments[0].end.y, 0.0);

		// The second ring's last segment: (3,3)->(2,3)
		let last_seg = &segments[4];
		assert_eq!(last_seg.start.x, 3.0);
		assert_eq!(last_seg.start.y, 3.0);
		assert_eq!(last_seg.end.x, 2.0);
		assert_eq!(last_seg.end.y, 3.0);
	}

	#[test]
	fn test_contains_point() {
		let mut rings = Rings::new();

		// Make a simple square ring around the origin: (0,0)->(10,0)->(10,10)->(0,10)->(0,0)
		let mut ring1 = Ring::new();
		ring1.add_point(Point::new(0.0, 0.0));
		ring1.add_point(Point::new(10.0, 0.0));
		ring1.add_point(Point::new(10.0, 10.0));
		ring1.add_point(Point::new(0.0, 10.0));
		ring1.close();

		rings.add_ring(ring1);

		let inside = Point::new(5.0, 5.0);
		let outside = Point::new(11.0, 5.0);

		assert!(
			rings.contains_point(&inside),
			"Inside point should be contained."
		);
		assert!(
			!rings.contains_point(&outside),
			"Outside point should not be contained."
		);
	}

	#[test]
	fn test_contains_point_multiple_rings() {
		let mut rings = Rings::new();

		// Ring 1: a small square from (0,0) to (2,2)
		let mut ring1 = Ring::new();
		ring1.add_point(Point::new(0.0, 0.0));
		ring1.add_point(Point::new(2.0, 0.0));
		ring1.add_point(Point::new(2.0, 2.0));
		ring1.add_point(Point::new(0.0, 2.0));
		ring1.close();

		// Ring 2: a second square from (3,3) to (5,5)
		let mut ring2 = Ring::new();
		ring2.add_point(Point::new(3.0, 3.0));
		ring2.add_point(Point::new(5.0, 3.0));
		ring2.add_point(Point::new(5.0, 5.0));
		ring2.add_point(Point::new(3.0, 5.0));
		ring2.close();

		rings.add_ring(ring1);
		rings.add_ring(ring2);

		// Inside ring1
		let inside_ring1 = Point::new(1.0, 1.0);
		assert!(rings.contains_point(&inside_ring1));

		// Inside ring2
		let inside_ring2 = Point::new(4.0, 4.0);
		assert!(rings.contains_point(&inside_ring2));

		// Outside both
		let outside_all = Point::new(10.0, 10.0);
		assert!(!rings.contains_point(&outside_all));
	}
}
