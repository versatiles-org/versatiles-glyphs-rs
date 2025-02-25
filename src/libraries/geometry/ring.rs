use super::{BBox, Point, Segment};

pub struct Ring {
	pub points: Vec<Point>,
}

impl Ring {
	pub fn new() -> Self {
		Ring { points: Vec::new() }
	}

	pub fn is_empty(&self) -> bool {
		self.points.is_empty()
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
		if (first.x - last.x).abs() > f32::EPSILON || (first.y - last.y).abs() > f32::EPSILON {
			self.points.push(*first);
		}
	}

	pub fn get_bbox(&self) -> BBox {
		let mut bbox = BBox::new();
		for point in &self.points {
			bbox.include_point(point);
		}
		bbox
	}

	pub fn translate(&mut self, offset: Point) {
		for point in &mut self.points {
			point.translate(offset);
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
			.map(|(a, b)| Segment::new(*a, *b))
			.collect()
	}

	pub fn add_quadratic_bezier(
		&mut self,
		start: Point,
		ctrl: Point,
		end: Point,
		tolerance_sq: f32,
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
			self.add_quadratic_bezier(start, mid_1, mid, tolerance_sq);
			self.add_quadratic_bezier(mid, mid_2, end, tolerance_sq);
		}
	}

	pub fn add_cubic_bezier(
		&mut self,
		start: Point,
		c1: Point,
		c2: Point,
		end: Point,
		tolerance_sq: f32,
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
			self.add_cubic_bezier(start, p01, p012, mid, tolerance_sq);
			self.add_cubic_bezier(mid, p123, p23, end, tolerance_sq);
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
				if p2.y > pt.y {
					if is_left(&p1, p2, pt) > 0 {
						winding_number += 1;
					}
				}
			} else {
				if p2.y <= pt.y {
					if is_left(&p1, p2, pt) < 0 {
						winding_number -= 1;
					}
				}
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
