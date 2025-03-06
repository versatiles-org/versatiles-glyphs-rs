use super::{BBox, Point, Segment};

#[derive(Clone, Debug, PartialEq)]
pub struct Ring {
	pub points: Vec<Point>,
}

impl Default for Ring {
	fn default() -> Self {
		Self::new()
	}
}

impl Ring {
	pub fn new() -> Self {
		Ring { points: Vec::new() }
	}

	pub fn is_empty(&self) -> bool {
		self.points.is_empty()
	}

	pub fn len(&self) -> usize {
		self.points.len()
	}

	pub fn clear(&mut self) {
		self.points.clear();
	}

	pub fn add_point(&mut self, point: Point) {
		self.points.push(point);
	}

	pub fn close(&mut self) {
		if self.points.is_empty() {
			return;
		}

		let first = self.points.first().unwrap();
		let last = self.points.last().unwrap();
		if (first.x - last.x).abs() > f64::EPSILON || (first.y - last.y).abs() > f64::EPSILON {
			self.points.push(first.clone());
		}
	}

	pub fn get_bbox(&self) -> BBox {
		let mut bbox = BBox::new();
		for point in &self.points {
			bbox.include_point(point);
		}
		bbox
	}

	pub fn translate(&mut self, offset: &Point) {
		for point in &mut self.points {
			point.translate(offset);
		}
	}

	pub fn scale(&mut self, scale: f64) {
		for point in &mut self.points {
			point.scale(scale);
		}
	}

	pub fn last(&self) -> Option<&Point> {
		self.points.last()
	}

	pub fn get_segments(&self) -> Vec<Segment> {
		self
			.points
			.iter()
			.zip(self.points.iter().skip(1))
			.map(|(a, b)| Segment::new(a, b))
			.collect()
	}

	pub fn add_quadratic_bezier(
		&mut self,
		start: &Point,
		ctrl: &Point,
		end: Point,
		tolerance_sq: f64,
	) {
		// Evaluate midpoints
		let mid_1 = start.midpoint(&ctrl);
		let mid_2 = ctrl.midpoint(&end);
		let mid = mid_1.midpoint(&mid_2);

		// We check if the curve is "flat enough"
		let dx = start.x + end.x - ctrl.x * 2.0;
		let dy = start.y + end.y - ctrl.y * 2.0;
		let dist_sq = dx * dx + dy * dy;

		if dist_sq <= tolerance_sq {
			// It's flat enough, just line to the end
			self.add_point(end);
		} else {
			// Subdivide
			self.add_quadratic_bezier(start, &mid_1, mid.clone(), tolerance_sq);
			self.add_quadratic_bezier(&mid, &mid_2, end, tolerance_sq);
		}
	}

	pub fn add_cubic_bezier(
		&mut self,
		start: &Point,
		c1: &Point,
		c2: &Point,
		end: Point,
		tolerance_sq: f64,
	) {
		// Using De Casteljau or similar approach.
		// Compute midpoints
		let p01 = start.midpoint(&c1);
		let p12 = c1.midpoint(&c2);
		let p23 = c2.midpoint(&end);
		let p012 = p01.midpoint(&p12);
		let p123 = p12.midpoint(&p23);
		let mid = p012.midpoint(&p123);

		// Check "flatness" by approximating the distance from midpoints
		let dx = (c2.x + c1.x) - (start.x + end.x);
		let dy = (c2.y + c1.y) - (start.y + end.y);
		let dist_sq = dx * dx + dy * dy;

		if dist_sq <= tolerance_sq {
			// Flat enough
			self.add_point(end);
		} else {
			// Subdivide
			self.add_cubic_bezier(start, &p01, &p012, mid.clone(), tolerance_sq);
			self.add_cubic_bezier(&mid, &p123, &p23, end, tolerance_sq);
		}
	}

	pub fn winding_number(&self, pt: &Point) -> i32 {
		let ring = &self.points;
		if ring.len() < 2 {
			return 0;
		}
		let mut p1 = &ring[0];
		let mut winding_number = 0;
		for p2 in ring.iter().skip(1) {
			if p1.y <= pt.y {
				if p2.y > pt.y && is_left(p1, p2, pt) > 0 {
					winding_number += 1;
				}
			} else if p2.y <= pt.y && is_left(p1, p2, pt) < 0 {
				winding_number -= 1;
			}
			p1 = p2;
		}
		winding_number
	}
}

fn is_left(p0: &Point, p1: &Point, p2: &Point) -> i32 {
	let val = (p1.x - p0.x) * (p2.y - p0.y) - (p2.x - p0.x) * (p1.y - p0.y);
	if val > 0.0 {
		1
	} else if val < 0.0 {
		-1
	} else {
		0
	}
}

impl<T> From<Vec<T>> for Ring
where
	Point: From<T>,
{
	fn from(points: Vec<T>) -> Self {
		Ring {
			points: points.into_iter().map(|p| p.into()).collect(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	// ^ Adjust the `use` path as needed depending on how your project is structured.

	#[test]
	fn test_ring_new_is_empty() {
		let ring = Ring::new();
		assert!(ring.is_empty());
		assert_eq!(ring.points.len(), 0);
	}

	#[test]
	fn test_ring_add_point() {
		let mut ring = Ring::new();
		ring.add_point(Point::new(1.0, 2.0));
		assert!(!ring.is_empty());
		assert_eq!(ring.points.len(), 1);

		let p = ring.last().unwrap();
		assert_eq!(p.as_tuple(), (1.0, 2.0));
	}

	#[test]
	fn test_ring_close_adds_first_point_at_end() {
		let mut ring = Ring::new();
		ring.add_point(Point::new(0.0, 0.0));
		ring.add_point(Point::new(1.0, 1.0));
		assert_eq!(ring.points.len(), 2);

		ring.close();
		// Closing the ring should add the first point again if not already present
		assert_eq!(ring.points.len(), 3);
		assert_eq!(ring.points[0], ring.points[2]);
	}

	#[test]
	fn test_ring_close_does_not_duplicate_if_already_closed() {
		let mut ring = Ring::new();
		ring.add_point(Point::new(0.0, 0.0));
		ring.add_point(Point::new(1.0, 1.0));
		ring.add_point(Point::new(0.0, 0.0)); // Already closed by user
		assert_eq!(ring.points.len(), 3);

		ring.close();
		// Because the first and last points match, no extra point should be added
		assert_eq!(ring.points.len(), 3);
	}

	#[test]
	fn test_ring_get_bbox() {
		let mut ring = Ring::new();
		ring.add_point(Point::new(2.0, 3.0));
		ring.add_point(Point::new(-1.0, 10.0));
		ring.add_point(Point::new(5.0, -4.0));

		let bbox = ring.get_bbox();
		assert_eq!(bbox.min.as_tuple(), (-1.0, -4.0));
		assert_eq!(bbox.max.as_tuple(), (5.0, 10.0));
	}

	#[test]
	fn test_ring_translate() {
		let mut ring = Ring::new();
		ring.add_point(Point::new(0.0, 0.0));
		ring.add_point(Point::new(1.0, 2.0));

		ring.translate(&Point::new(3.0, 4.0));
		assert_eq!(ring.points[0].as_tuple(), (3.0, 4.0));
		assert_eq!(ring.points[1].as_tuple(), (4.0, 6.0));
	}

	#[test]
	fn test_ring_scale() {
		let mut ring = Ring::new();
		ring.add_point(Point::new(0.0, 0.0));
		ring.add_point(Point::new(1.0, 2.0));

		ring.scale(0.3);
		assert_eq!(ring.points[0].as_tuple(), (0.0, 0.0));
		assert_eq!(ring.points[1].as_tuple(), (0.3, 0.6));
	}

	#[test]
	fn test_ring_last() {
		let mut ring = Ring::new();
		assert!(ring.last().is_none());

		ring.add_point(Point::new(1.0, 2.0));
		let p = ring.last().unwrap();
		assert_eq!(p.as_tuple(), (1.0, 2.0));
	}

	#[test]
	fn test_ring_get_segments() {
		let mut ring = Ring::new();
		ring.add_point(Point::new(0.0, 0.0));
		ring.add_point(Point::new(10.0, 0.0));
		ring.add_point(Point::new(10.0, 5.0));
		// We have 3 points, so get_segments() should yield 2 segments:
		// 1) (0,0) -> (10,0)
		// 2) (10,0) -> (10,5)
		let segments = ring.get_segments();
		assert_eq!(segments.len(), 2);
		assert_eq!(segments[0].start.as_tuple(), (0.0, 0.0));
		assert_eq!(segments[0].end.as_tuple(), (10.0, 0.0));
		assert_eq!(segments[1].start.as_tuple(), (10.0, 0.0));
		assert_eq!(segments[1].end.as_tuple(), (10.0, 5.0));
	}

	#[test]
	fn test_ring_add_quadratic_bezier_flat() {
		// Start -> Ctrl -> End where the curve is a straight line if ctrl lies on line
		let mut ring = Ring::new();
		let start = Point::new(0.0, 0.0);
		let ctrl = Point::new(1.0, 0.0);
		let end = Point::new(2.0, 0.0);
		ring.add_point(start.clone());
		// A very large tolerance should force no recursion
		ring.add_quadratic_bezier(&start, &ctrl, end, 10000.0);

		// Because it's basically a flat line, it should add just end point
		// ring.points: [start, end]
		assert_eq!(ring.points.len(), 2);
		assert_eq!(ring.points[0].as_tuple(), (0.0, 0.0));
		assert_eq!(ring.points[1].as_tuple(), (2.0, 0.0));
	}

	#[test]
	fn test_ring_add_quadratic_bezier_subdiv() {
		// A real curve (ctrl is above the line start->end),
		// with a small tolerance so it subdivides.
		let mut ring = Ring::new();
		let start = Point::new(0.0, 0.0);
		let ctrl = Point::new(1.0, 2.0);
		let end = Point::new(2.0, 0.0);
		ring.add_point(start.clone());

		// A small tolerance => more subdivisions
		ring.add_quadratic_bezier(&start, &ctrl, end, 0.0001);

		// We won't test every single point,
		// but we can confirm we ended up with multiple points
		assert!(ring.points.len() > 2);
		// The last point should be "end"
		let last_point = ring.points.last().unwrap();
		assert!((last_point.x - 2.0).abs() < f64::EPSILON);
		assert!((last_point.y - 0.0).abs() < f64::EPSILON);
	}

	#[test]
	fn test_ring_add_cubic_bezier_flat() {
		let mut ring = Ring::new();
		let start = Point::new(0.0, 0.0);
		let c1 = Point::new(1.0, 0.0);
		let c2 = Point::new(2.0, 0.0);
		let end = Point::new(3.0, 0.0);
		ring.add_point(start.clone());

		ring.add_cubic_bezier(&start, &c1, &c2, end, 10000.0);
		// Because it's effectively a flat line, the ring should end with end
		assert_eq!(
			ring.points.len(),
			2,
			"No subdivisions expected if curve is flat"
		);
		let last = ring.points.last().unwrap();
		assert_eq!(last.x, 3.0);
		assert_eq!(last.y, 0.0);
	}

	#[test]
	fn test_ring_add_cubic_bezier_subdiv() {
		let mut ring = Ring::new();
		let start = Point::new(0.0, 0.0);
		let c1 = Point::new(0.0, 2.0);
		let c2 = Point::new(2.0, 2.0);
		let end = Point::new(2.0, 0.0);
		ring.add_point(start.clone());

		// Use a very small tolerance to force subdivisions
		ring.add_cubic_bezier(&start, &c1, &c2, end, 0.0001);

		// We expect more than 2 points in ring
		assert!(ring.points.len() > 2);
		// End point should be the last
		let last = ring.points.last().unwrap();
		assert!((last.x - 2.0).abs() < f64::EPSILON);
		assert!((last.y - 0.0).abs() < f64::EPSILON);
	}

	#[test]
	fn test_ring_winding_number_empty_or_single() {
		let ring = Ring::new();
		let pt = Point::new(1.0, 1.0);
		assert_eq!(ring.winding_number(&pt), 0);

		let mut ring2 = Ring::new();
		ring2.add_point(Point::new(0.0, 0.0));
		assert_eq!(ring2.winding_number(&pt), 0);

		// If there's only 2 points, it's basically just a line, not a closed polygon
		ring2.add_point(Point::new(10.0, 0.0));
		assert_eq!(ring2.winding_number(&pt), 0);
	}

	#[test]
	fn test_ring_winding_number_simple_square() {
		// A simple square ring around the origin:
		// (0,0) -> (10,0) -> (10,10) -> (0,10) -> back to (0,0)
		let mut ring = Ring::new();
		ring.add_point(Point::new(0.0, 0.0));
		ring.add_point(Point::new(10.0, 0.0));
		ring.add_point(Point::new(10.0, 10.0));
		ring.add_point(Point::new(0.0, 10.0));
		ring.close(); // ensures last point matches first

		let inside = Point::new(5.0, 5.0);
		let outside = Point::new(11.0, 5.0);

		// For a standard counterclockwise ring, any inside point
		// typically yields a winding_number of 1
		let wn_inside = ring.winding_number(&inside);
		assert_eq!(
			wn_inside, 1,
			"Expected winding number of 1 for inside point"
		);

		let wn_outside = ring.winding_number(&outside);
		assert_eq!(
			wn_outside, 0,
			"Expected winding number of 0 for outside point"
		);
	}

	#[test]
	fn test_is_left_function() {
		// Just to be explicit, though it's tested indirectly by winding_number
		let p0 = Point::new(0.0, 0.0);
		let p1 = Point::new(1.0, 0.0);
		// p2 above the line p0->p1 => is_left should return +1
		let p2_above = Point::new(0.5, 1.0);
		assert_eq!(is_left(&p0, &p1, &p2_above), 1);

		// p2 below the line => is_left should return -1
		let p2_below = Point::new(0.5, -1.0);
		assert_eq!(is_left(&p0, &p1, &p2_below), -1);

		// p2 exactly on the line => 0
		let p2_on_line = Point::new(0.5, 0.0);
		assert_eq!(is_left(&p0, &p1, &p2_on_line), 0);
	}
}
