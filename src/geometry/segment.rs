//! Defines a line segment between two [`Point`] references, providing methods for projecting
//! external points onto the segment and computing distance metrics.

use super::Point;

/// A line segment defined by references to two [`Point`]s (`start` and `end`).
///
/// This structure is useful for geometric operations such as point projection
/// and distance calculations.
#[derive(Clone, Debug)]
pub struct Segment<'a> {
	/// The start point of this segment.
	pub start: &'a Point,
	/// The end point of this segment.
	pub end: &'a Point,
}

impl<'a> Segment<'a> {
	/// Creates a new line segment from two references to [`Point`]s.
	///
	/// ```
	/// # use versatiles_glyphs::geometry::Point;
	/// # use versatiles_glyphs::geometry::Segment;
	/// let start = Point::new(1.0, 2.0);
	/// let end = Point::new(3.0, 4.0);
	/// let segment = Segment::new(&start, &end);
	/// assert_eq!(segment.start.x, 1.0);
	/// assert_eq!(segment.end.y, 4.0);
	/// ```
	pub fn new(start: &'a Point, end: &'a Point) -> Self {
		Segment { start, end }
	}

	/// Projects a given point `p` onto this segment and returns the resulting [`Point`].
	///
	/// If the segment is degenerate (start == end), the projection is the start point.  
	/// If the projection falls "before" the start or "after" the end (in terms of parametric t),
	/// the respective endpoint is returned.
	///
	/// # Examples
	///
	/// ```
	/// # use versatiles_glyphs::geometry::Point;
	/// # use versatiles_glyphs::geometry::Segment;
	/// let start = Point::new(0.0, 0.0);
	/// let end = Point::new(10.0, 0.0);
	/// let seg = Segment::new(&start, &end);
	///
	/// // A point above the midpoint at (5,5). Projecting onto the segment:
	/// let p = Point::new(5.0, 5.0);
	/// let projected = seg.project_point_on(&p);
	/// assert_eq!(projected.as_tuple(), (5.0, 0.0));
	/// ```
	pub fn project_point_on(&self, p: &Point) -> Point {
		let v = self.start;
		let w = self.end;
		let l2 = v.squared_distance_to(w);
		// If segment length is zero, return the single point.
		if l2 == 0.0 {
			return v.clone();
		}
		// Parametric t for the projection on the infinite line
		let t = ((p.x - v.x) * (w.x - v.x) + (p.y - v.y) * (w.y - v.y)) / l2;
		// If t < 0 or t > 1, it projects outside the segment, clamp to endpoints.
		if t < 0.0 {
			return v.clone();
		} else if t > 1.0 {
			return w.clone();
		}
		// Otherwise, project onto the interior
		Point::new(v.x + t * (w.x - v.x), v.y + t * (w.y - v.y))
	}

	/// Returns the squared distance from a given point `p` to this segment.
	///
	/// This first projects `p` onto the segment (via [`project_point_on`](Self::project_point_on))
	/// and then returns the squared distance from `p` to that projection.
	/// Squared distances are typically used to avoid computing expensive square roots
	/// when only comparisons are needed.
	///
	/// # Examples
	///
	/// ```
	/// # use versatiles_glyphs::geometry::Point;
	/// # use versatiles_glyphs::geometry::Segment;
	/// let start = Point::new(0.0, 0.0);
	/// let end = Point::new(5.0, 0.0);
	/// let seg = Segment::new(&start, &end);
	///
	/// let p = Point::new(0.0, 3.0);
	/// let dist_sq = seg.squared_distance_to_point(&p);
	/// // The nearest point is (0, 0), so squared distance is 3^2 = 9.0
	/// assert!((dist_sq - 9.0).abs() < f64::EPSILON);
	/// ```
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

		let external_p = Point::new(10.0, 10.0);
		let proj = degenerate_seg.project_point_on(&external_p);
		assert_eq!(proj.x, degenerate_seg.start.x);
		assert_eq!(proj.y, degenerate_seg.start.y);
	}

	#[test]
	fn test_project_point_on_segment_before_start() {
		let start = Point::new(1.0, 1.0);
		let end = Point::new(5.0, 1.0);
		let seg = Segment::new(&start, &end);

		let p = Point::new(-2.0, 1.0);
		let proj = seg.project_point_on(&p);
		assert_eq!(proj.x, 1.0);
		assert_eq!(proj.y, 1.0);
	}

	#[test]
	fn test_project_point_on_segment_after_end() {
		let start = Point::new(1.0, 1.0);
		let end = Point::new(5.0, 1.0);
		let seg = Segment::new(&start, &end);

		let p = Point::new(10.0, 1.0);
		let proj = seg.project_point_on(&p);
		assert_eq!(proj.x, 5.0);
		assert_eq!(proj.y, 1.0);
	}

	#[test]
	fn test_project_point_on_segment_in_between() {
		let start = Point::new(0.0, 0.0);
		let end = Point::new(10.0, 0.0);
		let seg = Segment::new(&start, &end);

		let p = Point::new(5.0, 5.0);
		let proj = seg.project_point_on(&p);
		assert_eq!(proj.x, 5.0);
		assert_eq!(proj.y, 0.0);
	}

	#[test]
	fn test_project_point_on_segment_diagonal() {
		let start = Point::new(0.0, 0.0);
		let end = Point::new(4.0, 4.0);
		let seg = Segment::new(&start, &end);

		let p = Point::new(2.0, 3.0);
		let proj = seg.project_point_on(&p);
		assert!((proj.x - 2.5).abs() < f64::EPSILON);
		assert!((proj.y - 2.5).abs() < f64::EPSILON);
	}

	#[test]
	fn test_squared_distance_to_point() {
		let start = Point::new(0.0, 0.0);
		let end = Point::new(5.0, 0.0);
		let seg = Segment::new(&start, &end);

		// nearest point => (0,0)
		let p = Point::new(0.0, 3.0);
		let dist_sq = seg.squared_distance_to_point(&p);
		assert!((dist_sq - 9.0).abs() < f64::EPSILON);

		// nearest point => (5,0)
		let p_far = Point::new(10.0, 0.0);
		let dist_sq_far = seg.squared_distance_to_point(&p_far);
		assert!((dist_sq_far - 25.0).abs() < f64::EPSILON);

		// nearest point => (2,0)
		let p_above = Point::new(2.0, 4.0);
		let dist_sq_above = seg.squared_distance_to_point(&p_above);
		assert!((dist_sq_above - 16.0).abs() < f64::EPSILON);
	}
}
