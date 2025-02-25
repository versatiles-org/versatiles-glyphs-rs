use super::super::geometry::{Point, Ring, Rings};
use std::mem::swap;
use ttf_parser::OutlineBuilder;

pub struct RingBuilder {
	rings: Rings,
	ring: Ring,
}

impl RingBuilder {
	pub fn new() -> Self {
		RingBuilder {
			rings: Rings::new(),
			ring: Ring::new(),
		}
	}

	pub fn into_rings(mut self) -> Rings {
		self.save_ring();
		self.rings
	}

	fn save_ring(&mut self) {
		if self.ring.len() < 3 {
			self.ring.clear();
			return;
		}

		self.ring.close();
		if self.ring.len() < 4 {
			self.ring.clear();
			return;
		}

		let mut ring = Ring::new();
		swap(&mut self.ring, &mut ring);
		self.rings.add_ring(ring);
	}
}

impl OutlineBuilder for RingBuilder {
	fn move_to(&mut self, x: f32, y: f32) {
		self.save_ring();
		self.ring.add_point(Point::new(x, y));
	}

	fn line_to(&mut self, x: f32, y: f32) {
		self.ring.add_point(Point::new(x, y));
	}

	fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
		if self.ring.is_empty() {
			return;
		}
		self.ring.add_quadratic_bezier(
			*self.ring.last().unwrap(),
			Point::new(x1, y1),
			Point::new(x, y),
			0.3,
		);
	}

	fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
		if self.ring.is_empty() {
			return;
		}
		self.ring.add_cubic_bezier(
			*self.ring.last().unwrap(),
			Point::new(x1, y1),
			Point::new(x2, y2),
			Point::new(x, y),
			0.3,
		);
	}

	fn close(&mut self) {
		self.save_ring();
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ring_builder_new() {
		let builder = RingBuilder::new();
		// Initially, there's one empty ring inside
		assert!(builder.ring.is_empty());
		// The Rings collection is empty
		// (though builder.rings exists, it has zero actual rings added yet)
		assert_eq!(builder.rings.get_segments().len(), 0);
	}

	#[test]
	fn test_drop_invalid_ring() {
		let mut builder = RingBuilder::new();
		builder.move_to(10.0, 20.0);
		// Now the builder's current ring has one point
		assert_eq!(builder.rings.len(), 0);
		assert_eq!(builder.ring.len(), 1);
		assert_eq!(builder.ring.points[0].as_tuple(), (10.0, 20.0));

		// If we call move_to again, we save the previous ring and start a new one
		builder.move_to(30.0, 40.0);
		// The old ring has been dropped
		assert_eq!(builder.rings.len(), 0);
		assert_eq!(builder.ring.len(), 1);
		assert_eq!(builder.ring.points[0].as_tuple(), (30.0, 40.0));
	}

	#[test]
	fn test_line_to() {
		let mut builder = RingBuilder::new();
		builder.move_to(0.0, 0.0);
		builder.line_to(1.0, 2.0);
		builder.line_to(-1.0, 3.0);

		// Now the current ring should have two points
		assert_eq!(builder.ring.len(), 3);
		assert_eq!(builder.rings.len(), 0);
		assert_eq!(builder.ring.points[0].as_tuple(), (0.0, 0.0));
		assert_eq!(builder.ring.points[1].as_tuple(), (1.0, 2.0));
		assert_eq!(builder.ring.points[2].as_tuple(), (-1.0, 3.0));

		// Checking ring segments if we forcibly save the ring
		builder.close(); // close does not finalize the ring to Rings; it just closes it

		assert_eq!(builder.ring.len(), 0);
		assert_eq!(builder.rings.len(), 1);

		// The ring is now in builder.rings
		// We can check how many segments we have:
		let segments = builder.into_rings().get_segments();
		assert_eq!(
			segments.len(),
			3,
			"Expected 3 line segments from move_to->line_to."
		);
		assert_eq!(segments[0].start.as_tuple(), (0.0, 0.0));
		assert_eq!(segments[0].end.as_tuple(), (1.0, 2.0));
		assert_eq!(segments[1].start.as_tuple(), (1.0, 2.0));
		assert_eq!(segments[1].end.as_tuple(), (-1.0, 3.0));
		assert_eq!(segments[2].start.as_tuple(), (-1.0, 3.0));
		assert_eq!(segments[2].end.as_tuple(), (0.0, 0.0));
	}

	#[test]
	fn test_quad_to_when_empty_does_nothing() {
		let mut builder = RingBuilder::new();
		// We don't move_to or line_to first, so the ring is empty
		builder.quad_to(10.0, 10.0, 20.0, 20.0);
		// Because ring is empty, quad_to should do nothing
		assert!(builder.ring.is_empty());
	}

	#[test]
	fn test_quad_to_adds_curve_if_not_empty() {
		let mut builder = RingBuilder::new();
		builder.move_to(0.0, 0.0);
		builder.quad_to(10.0, 10.0, 20.0, 0.0);
		// The ring should now contain the subdivided curve points
		// The first point is (0,0). Then several points for the subdivided curve,
		// ending in (20,0). The subdiv logic in `add_quadratic_bezier` depends on tolerance.
		// Hard to precisely match the count, but we can check the last point:

		let ring_len = builder.ring.points.len();
		assert!(
			ring_len >= 2,
			"Ring should have at least start and end points after quad_to"
		);
		let last_point = builder.ring.points.last().unwrap();
		assert!(
			(last_point.x - 20.0).abs() < f32::EPSILON,
			"Expected end x=20.0"
		);
		assert!(
			(last_point.y - 0.0).abs() < f32::EPSILON,
			"Expected end y=0.0"
		);
	}

	#[test]
	fn test_curve_to_when_empty_does_nothing() {
		let mut builder = RingBuilder::new();
		builder.curve_to(10.0, 10.0, 20.0, 20.0, 30.0, 0.0);
		// Because ring is empty, curve_to should do nothing
		assert!(builder.ring.is_empty());
	}

	#[test]
	fn test_curve_to_adds_curve_if_not_empty() {
		let mut builder = RingBuilder::new();
		builder.move_to(0.0, 0.0);
		builder.curve_to(10.0, 10.0, 20.0, 10.0, 30.0, 0.0);
		// The ring should contain subdivided cubic points from (0,0) to (30,0).

		let ring_len = builder.ring.points.len();
		assert!(
			ring_len >= 2,
			"Ring should have at least start and end after curve_to"
		);
		let last_point = builder.ring.points.last().unwrap();
		assert!(
			(last_point.x - 30.0).abs() < f32::EPSILON,
			"Expected end x=30.0"
		);
		assert!(
			(last_point.y - 0.0).abs() < f32::EPSILON,
			"Expected end y=0.0"
		);
	}

	#[test]
	fn test_close_closes_current_ring() {
		let mut builder = RingBuilder::new();
		builder.move_to(0.0, 0.0);
		builder.line_to(10.0, 0.0);
		builder.line_to(20.0, 0.0);

		// Before close, ring has 2 points
		assert_eq!(builder.ring.len(), 3);
		assert_eq!(builder.rings.len(), 0);
		builder.close();
		// After close, the ring might have the first point repeated
		// if the ring wasn't degenerate. So we expect ring.points to have 3 points
		// if the first != last. Here the first != last.
		assert_eq!(builder.ring.len(), 0);
		assert_eq!(builder.rings.len(), 1);
		assert_eq!(builder.rings.rings[0].len(), 4);

		let ring = &builder.rings.rings[0];
		let first = ring.points[0];
		let last = ring.points.last().unwrap();
		assert_eq!(first.squared_distance_to(last), 0.0);
	}

	#[test]
	fn test_into_rings_moves_current_ring() {
		let mut builder = RingBuilder::new();
		builder.move_to(0.0, 0.0);
		builder.line_to(1.0, 0.0);
		builder.line_to(0.0, 2.0);

		// We have not yet saved or closed the ring to the rings collection.
		assert_eq!(builder.ring.len(), 3);
		assert_eq!(builder.rings.len(), 0);

		// Now convert into rings
		let rings = builder.into_rings();

		// The ring with the line segment is in `rings`
		let segments = rings.get_segments();
		assert_eq!(segments.len(), 3, "Expected 3 line segments in final rings");
	}
}
