/// A simple 2D point with `x` and `y` coordinates.
///
/// This struct includes convenience methods for geometric transformations
/// (translation, inversion, scaling) and measuring distances.
#[derive(Clone, Debug, PartialEq)]
pub struct Point {
	/// The x-coordinate.
	pub x: f64,
	/// The y-coordinate.
	pub y: f64,
}

impl Point {
	/// Creates a new [`Point`] from the given `x` and `y`.
	pub fn new(x: f64, y: f64) -> Self {
		Point { x, y }
	}

	/// Returns the midpoint between `self` and another [`Point`].
	///
	/// ```
	/// # use versatiles_glyphs::geometry::point::Point;
	/// let p1 = Point::new(0.0, 0.0);
	/// let p2 = Point::new(4.0, 6.0);
	/// let mid = p1.midpoint(&p2);
	/// assert_eq!(mid, Point::new(2.0, 3.0));
	/// ```
	#[inline(always)]
	pub fn midpoint(&self, other: &Point) -> Self {
		Point::new((self.x + other.x) / 2.0, (self.y + other.y) / 2.0)
	}

	/// Returns the squared distance between `self` and another [`Point`].
	///
	/// This avoids computing a square root, making it efficient for
	/// distance comparisons.
	#[inline(always)]
	pub fn squared_distance_to(&self, other: &Point) -> f64 {
		let dx = other.x - self.x;
		let dy = other.y - self.y;
		dx * dx + dy * dy
	}

	/// Returns a copy of this [`Point`] with its coordinates inverted (`-x`, `-y`).
	///
	/// ```
	/// # use versatiles_glyphs::geometry::point::Point;
	/// let p = Point::new(2.0, -3.0).inverted();
	/// assert_eq!(p, Point::new(-2.0, 3.0));
	/// ```
	pub fn inverted(mut self) -> Self {
		self.x = -self.x;
		self.y = -self.y;
		self
	}

	/// Returns a new [`Point`] by translating `self` with an offset, consuming `self`.
	///
	/// ```
	/// # use versatiles_glyphs::geometry::point::Point;
	/// let original = Point::new(1.0, 2.0);
	/// let offset = Point::new(3.5, -0.5);
	/// let new_p = original.translated(&offset);
	/// assert_eq!(new_p, Point::new(4.5, 1.5));
	/// ```
	pub fn translated(mut self, offset: &Point) -> Self {
		self.x += offset.x;
		self.y += offset.y;
		self
	}

	/// Translates this [`Point`] in place.
	///
	/// ```
	/// # use versatiles_glyphs::geometry::point::Point;
	/// let mut p = Point::new(2.0, 3.0);
	/// let offset = Point::new(-2.0, 1.0);
	/// p.translate(&offset);
	/// assert_eq!(p, Point::new(0.0, 4.0));
	/// ```
	pub fn translate(&mut self, offset: &Point) {
		self.x += offset.x;
		self.y += offset.y;
	}

	/// Scales this [`Point`] in place by the specified factor.
	///
	/// ```
	/// # use versatiles_glyphs::geometry::point::Point;
	/// let mut p = Point::new(2.0, 3.0);
	/// p.scale(4.0);
	/// assert_eq!(p, Point::new(8.0, 12.0));
	/// ```
	pub fn scale(&mut self, scale: f64) {
		self.x *= scale;
		self.y *= scale;
	}

	/// Returns the coordinates of this [`Point`] as a tuple.
	pub fn as_tuple(&self) -> (f64, f64) {
		(self.x, self.y)
	}
}

impl From<(f32, f32)> for Point {
	fn from(t: (f32, f32)) -> Self {
		Point::new(t.0 as f64, t.1 as f64)
	}
}

impl From<(f64, f64)> for Point {
	fn from(t: (f64, f64)) -> Self {
		Point::new(t.0, t.1)
	}
}

impl From<(i32, i32)> for Point {
	fn from(t: (i32, i32)) -> Self {
		Point::new(t.0 as f64, t.1 as f64)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_point_new() {
		let p = Point::new(3.0, -4.5);
		assert_eq!(p.as_tuple(), (3.0, -4.5));
	}

	#[test]
	fn test_point_midpoint() {
		let p1 = Point::new(0.0, 0.0);
		let p2 = Point::new(4.0, 6.0);
		let midpoint = p1.midpoint(&p2);
		assert_eq!(midpoint.as_tuple(), (2.0, 3.0));
	}

	#[test]
	fn test_point_squared_distance_to() {
		let p1 = Point::new(1.0, 1.0);
		let p2 = Point::new(4.0, 5.0);
		assert_eq!(p1.squared_distance_to(&p2), 25.0);
		assert_eq!(p2.squared_distance_to(&p1), 25.0);
	}

	#[test]
	fn test_point_inverted() {
		let p = Point::new(2.0, -3.0).inverted();
		assert_eq!(p.as_tuple(), (-2.0, 3.0));
	}

	#[test]
	fn test_point_translated_consuming() {
		let original = Point::new(1.0, 2.0);
		let offset = Point::new(3.5, -0.5);
		let new_p = original.translated(&offset);
		assert_eq!(new_p.as_tuple(), (4.5, 1.5));
	}

	#[test]
	fn test_point_translate_in_place() {
		let mut p = Point::new(2.0, 3.0);
		let offset = Point::new(-2.0, 1.0);
		p.translate(&offset);
		assert_eq!(p.as_tuple(), (0.0, 4.0));
	}

	#[test]
	fn test_point_scale() {
		let mut p = Point::new(2.0, 3.0);
		p.scale(4.0);
		assert_eq!(p.as_tuple(), (8.0, 12.0));
	}
}
