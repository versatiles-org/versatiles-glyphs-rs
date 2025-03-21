use crate::geometry::{Point, Ring, Rings};
use std::mem::swap;
use ttf_parser::OutlineBuilder;

/// Implements [`ttf_parser::OutlineBuilder`] to accumulate glyph outline data
/// into a [`Rings`] collection. Each ring represents a continuous path of
/// points (lines or Bezier segments) in the glyph outline.
pub struct RingBuilder {
	/// The collection of all rings built so far.
	rings: Rings,

	/// The current active ring, being populated as outline commands arrive.
	ring: Ring,

	/// The approximation precision for Bezier curves. A smaller value increases
	/// the number of line segments approximating each curve.
	precision: f64,
}

impl RingBuilder {
	/// Finalizes the current ring (if valid) and returns all built rings.
	///
	/// This method will automatically close and save the active ring
	/// before returning the [`Rings`]. Once called, this builder should
	/// no longer be used.
	pub fn into_rings(mut self) -> Rings {
		self.save_ring();
		self.rings
	}

	/// Closes and validates the current ring, adds it to the collection
	/// if it has enough points, then starts a fresh ring.
	fn save_ring(&mut self) {
		// Ignore any ring with fewer than 3 points (not a valid polygon).
		if self.ring.len() < 3 {
			self.ring.clear();
			return;
		}

		// Close the ring by linking its last point to its first,
		// ensuring it forms a proper loop.
		self.ring.close();

		// If, after closing, it's too short, discard it.
		if self.ring.len() < 4 {
			self.ring.clear();
			return;
		}

		// Swap out the current ring and store it in `rings`.
		let mut ring = Ring::new();
		swap(&mut self.ring, &mut ring);
		self.rings.add_ring(ring);
	}
}

impl Default for RingBuilder {
	fn default() -> Self {
		RingBuilder {
			rings: Rings::new(),
			ring: Ring::new(),
			precision: 0.01,
		}
	}
}

impl OutlineBuilder for RingBuilder {
	/// Moves the drawing cursor to `(x, y)`, closing any currently active ring.
	fn move_to(&mut self, x: f32, y: f32) {
		self.save_ring();
		self.ring.add_point(Point::from((x, y)));
	}

	/// Draws a straight line from the current cursor to `(x, y)`.
	fn line_to(&mut self, x: f32, y: f32) {
		self.ring.add_point(Point::from((x, y)));
	}

	/// Draws a quadratic Bézier curve from the current cursor position
	/// to `(x, y)`, using `(x1, y1)` as the control point. The curve
	/// is approximated via line segments based on the builder's precision.
	fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
		if self.ring.is_empty() {
			return;
		}
		let start = self.ring.last().unwrap().clone();
		self.ring.add_quadratic_bezier(
			&start,
			&Point::from((x1, y1)),
			Point::from((x, y)),
			self.precision,
		);
	}

	/// Draws a cubic Bézier curve from the current cursor position to `(x, y)`,
	/// using `(x1, y1)` and `(x2, y2)` as control points. The curve is approximated
	/// via line segments based on the builder's precision.
	fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
		if self.ring.is_empty() {
			return;
		}
		let start = self.ring.last().unwrap().clone();
		self.ring.add_cubic_bezier(
			&start,
			&Point::from((x1, y1)),
			&Point::from((x2, y2)),
			Point::from((x, y)),
			self.precision,
		);
	}

	/// Closes the current ring by connecting its last point back to the first.
	/// If the ring is valid (at least 3 points), it is saved to the collection.
	fn close(&mut self) {
		self.save_ring();
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_ring_builder_new() {
		let builder = RingBuilder::default();
		// Initially, there's one empty ring inside
		assert!(builder.ring.is_empty());
		// The Rings collection is empty
		// (though builder.rings exists, it has zero actual rings added yet)
		assert_eq!(builder.rings.get_segments().len(), 0);
	}

	#[test]
	fn test_drop_invalid_ring() {
		let mut builder = RingBuilder::default();
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
		let mut builder = RingBuilder::default();
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
		let rings = builder.into_rings();
		let segments = rings.get_segments();
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
		let mut builder = RingBuilder::default();
		// We don't move_to or line_to first, so the ring is empty
		builder.quad_to(10.0, 10.0, 20.0, 20.0);
		// Because ring is empty, quad_to should do nothing
		assert!(builder.ring.is_empty());
	}

	#[test]
	fn test_quad_to_adds_curve_if_not_empty() {
		let mut builder = RingBuilder::default();
		builder.move_to(0.0, 0.0);
		builder.quad_to(10.0, 10.0, 20.0, 0.0);
		// The ring should now contain the subdivided curve points
		// The first point is (0,0). Then several points for the subdivided curve,
		// ending in (20,0). The subdiv logic in `add_quadratic_bezier` depends on tolerance.
		// Hard to precisely match the count, but we can check the last point:

		assert_eq!(builder.ring.points.len(), 17);
		let last_point = builder.ring.points.last().unwrap();
		assert_eq!(last_point.as_tuple(), (20.0, 0.0));
	}

	#[test]
	fn test_curve_to_when_empty_does_nothing() {
		let mut builder = RingBuilder::default();
		builder.curve_to(10.0, 10.0, 20.0, 20.0, 30.0, 0.0);
		// Because ring is empty, curve_to should do nothing
		assert!(builder.ring.is_empty());
	}

	#[test]
	fn test_curve_to_adds_curve_if_not_empty() {
		let mut builder = RingBuilder::default();
		builder.move_to(0.0, 0.0);
		builder.curve_to(10.0, 10.0, 20.0, 10.0, 30.0, 0.0);
		// The ring should contain subdivided cubic points from (0,0) to (30,0).

		assert_eq!(builder.ring.points.len(), 17);
		let last_point = builder.ring.points.last().unwrap();
		assert_eq!(last_point.as_tuple(), (30.0, 0.0));
	}

	#[test]
	fn test_close_closes_current_ring() {
		let mut builder = RingBuilder::default();
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
		let first = &ring.points[0];
		let last = ring.points.last().unwrap();
		assert_eq!(first.squared_distance_to(last), 0.0);
	}

	#[test]
	fn test_into_rings_moves_current_ring() {
		let mut builder = RingBuilder::default();
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
		assert_eq!(segments.len(), 3);
	}
}
