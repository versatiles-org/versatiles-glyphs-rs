use super::Point;

// A line segment is just two Points
#[derive(Clone, Debug)]
pub struct Segment<'a> {
	pub start: &'a Point,
	pub end: &'a Point,
}

impl<'a> Segment<'a> {
	pub fn new(start: &'a Point, end: &'a Point) -> Self {
		Segment { start, end }
	}

	pub fn project_point_on(&self, p: &Point) -> Point {
		let v = self.start;
		let w = self.end;
		let l2 = v.squared_distance_to(&w);
		if l2 == 0.0 {
			return v.clone();
		}
		let t = ((p.x - v.x) * (w.x - v.x) + (p.y - v.y) * (w.y - v.y)) / l2;
		if t < 0.0 {
			return v.clone();
		} else if t > 1.0 {
			return w.clone();
		}
		Point::new(v.x + t * (w.x - v.x), v.y + t * (w.y - v.y))
	}

	#[inline(always)]
	pub fn squared_distance_to_point(&self, p: &Point) -> f64 {
		let proj = self.project_point_on(p);
		p.squared_distance_to(&proj)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_segment_new() {
		let start = Point::new(1.0, 2.0);
		let end = Point::new(3.0, 4.0);
		let segment = Segment::new(&start, &end);
		assert_eq!(segment.start.x, 1.0);
		assert_eq!(segment.start.y, 2.0);
		assert_eq!(segment.end.x, 3.0);
		assert_eq!(segment.end.y, 4.0);
	}

	#[test]
	fn test_project_point_on_zero_length_segment() {
		// If start == end, the segment is a degenerate line (a single point).
		let start = Point::new(2.0, 3.0);
		let end = Point::new(2.0, 3.0);
		let degenerate_seg = Segment::new(&start, &end);
		// The projection of any point onto this "segment" should return that single point.
		let external_p = Point::new(10.0, 10.0);
		let proj = degenerate_seg.project_point_on(&external_p);
		assert_eq!(proj.x, degenerate_seg.start.x);
		assert_eq!(proj.y, degenerate_seg.start.y);
	}

	#[test]
	fn test_project_point_on_segment_before_start() {
		// Segment from (1, 1) to (5, 1). A horizontal segment.
		let start = Point::new(1.0, 1.0);
		let end = Point::new(5.0, 1.0);
		let seg = Segment::new(&start, &end);
		// A point far "to the left" (i.e., x < 1)
		let p = Point::new(-2.0, 1.0);
		let proj = seg.project_point_on(&p);
		// Because it's "before start" along the segment direction, projection should be start = (1,1).
		assert_eq!(proj.x, 1.0);
		assert_eq!(proj.y, 1.0);
	}

	#[test]
	fn test_project_point_on_segment_after_end() {
		// Segment from (1, 1) to (5, 1).
		let start = Point::new(1.0, 1.0);
		let end = Point::new(5.0, 1.0);
		let seg = Segment::new(&start, &end);
		// A point far "to the right" (i.e., x > 5)
		let p = Point::new(10.0, 1.0);
		let proj = seg.project_point_on(&p);
		// Because it's "past end" along the segment direction, projection should be end = (5,1).
		assert_eq!(proj.x, 5.0);
		assert_eq!(proj.y, 1.0);
	}

	#[test]
	fn test_project_point_on_segment_in_between() {
		// Segment from (0, 0) to (10, 0). A horizontal segment along x.
		let start = Point::new(0.0, 0.0);
		let end = Point::new(10.0, 0.0);
		let seg = Segment::new(&start, &end);
		// A point above the midpoint, e.g., (5, 5).
		let p = Point::new(5.0, 5.0);
		let proj = seg.project_point_on(&p);
		// The projection onto this segment is (5, 0).
		assert_eq!(proj.x, 5.0);
		assert_eq!(proj.y, 0.0);
	}

	#[test]
	fn test_project_point_on_segment_diagonal() {
		// Diagonal segment from (0, 0) to (4, 4)
		let start = Point::new(0.0, 0.0);
		let end = Point::new(4.0, 4.0);
		let seg = Segment::new(&start, &end);
		// Point (2, 3) is not directly along line y=x, so let's see projection.
		// The param t is given by dot((p - v), (w - v)) / |w - v|^2
		// (w - v) = (4,4), (p - v) = (2,3).
		// dot( (2,3), (4,4) ) = 2*4 + 3*4 = 8 + 12 = 20
		// |w - v|^2 = 4^2 + 4^2 = 16 + 16 = 32
		// t = 20/32 = 0.625
		// Then projection = (0,0) + 0.625*(4,4) = (2.5, 2.5)
		let p = Point::new(2.0, 3.0);
		let proj = seg.project_point_on(&p);
		assert!(
			(proj.x - 2.5).abs() < f64::EPSILON,
			"Expected 2.5 but got {}",
			proj.x
		);
		assert!(
			(proj.y - 2.5).abs() < f64::EPSILON,
			"Expected 2.5 but got {}",
			proj.y
		);
	}

	#[test]
	fn test_squared_distance_to_point() {
		// Segment from (0, 0) to (5, 0)
		let start = Point::new(0.0, 0.0);
		let end = Point::new(5.0, 0.0);
		let seg = Segment::new(&start, &end);
		// A point (0, 3) is to the left (on the y-axis), and nearest segment point is (0,0).
		// squared distance should be 9
		let p = Point::new(0.0, 3.0);
		let dist_sq = seg.squared_distance_to_point(&p);
		assert!(
			(dist_sq - 9.0).abs() < f64::EPSILON,
			"Expected 9.0 but got {}",
			dist_sq
		);

		// A point (10, 0) is beyond the end of the segment. Nearest point is (5, 0).
		// squared distance is (10-5)^2 + (0-0)^2 = 25
		let p_far = Point::new(10.0, 0.0);
		let dist_sq_far = seg.squared_distance_to_point(&p_far);
		assert!(
			(dist_sq_far - 25.0).abs() < f64::EPSILON,
			"Expected 25.0 but got {}",
			dist_sq_far
		);

		// A point (2, 4) is above the segment. The nearest point is (2, 0).
		// squared distance is (2-2)^2 + (4-0)^2 = 16
		let p_above = Point::new(2.0, 4.0);
		let dist_sq_above = seg.squared_distance_to_point(&p_above);
		assert!(
			(dist_sq_above - 16.0).abs() < f64::EPSILON,
			"Expected 16.0 but got {}",
			dist_sq_above
		);
	}
}
