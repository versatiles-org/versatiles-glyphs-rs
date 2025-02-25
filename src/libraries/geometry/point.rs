#[derive(Clone, Copy, Debug)]
pub struct Point {
	pub x: f32,
	pub y: f32,
}

impl Point {
	pub fn new(x: f32, y: f32) -> Self {
		Point { x, y }
	}

	pub fn midpoint(&self, other: &Point) -> Self {
		Point::new((self.x + other.x) / 2.0, (self.y + other.y) / 2.0)
	}

	pub fn squared_distance_to(&self, other: &Point) -> f32 {
		let dx = other.x - self.x;
		let dy = other.y - self.y;
		dx * dx + dy * dy
	}

	pub fn inverted(mut self) -> Self {
		self.x = -self.x;
		self.y = -self.y;
		self
	}

	pub fn translated(mut self, offset: Point) -> Self {
		self.x += offset.x;
		self.y += offset.y;
		self
	}

	pub fn translate(&mut self, offset: Point) {
		self.x += offset.x;
		self.y += offset.y;
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_point_new() {
		let p = Point::new(3.0, -4.5);
		assert_eq!(p.x, 3.0);
		assert_eq!(p.y, -4.5);
	}

	#[test]
	fn test_point_midpoint() {
		let p1 = Point::new(0.0, 0.0);
		let p2 = Point::new(4.0, 6.0);
		let midpoint = p1.midpoint(&p2);
		// midpoint = ((0 + 4)/2, (0 + 6)/2) = (2, 3)
		assert_eq!(midpoint.x, 2.0);
		assert_eq!(midpoint.y, 3.0);
	}

	#[test]
	fn test_point_squared_distance_to() {
		let p1 = Point::new(1.0, 1.0);
		let p2 = Point::new(4.0, 5.0);
		// dx = 3, dy = 4 => squared distance = 9 + 16 = 25
		assert_eq!(p1.squared_distance_to(&p2), 25.0);
		// Should be symmetric
		assert_eq!(p2.squared_distance_to(&p1), 25.0);
	}

	#[test]
	fn test_point_inverted() {
		let p = Point::new(2.0, -3.0).inverted();
		// Inverted => (-2, 3)
		assert_eq!(p.x, -2.0);
		assert_eq!(p.y, 3.0);
	}

	#[test]
	fn test_point_translated_consuming() {
		let original = Point::new(1.0, 2.0);
		let offset = Point::new(3.5, -0.5);
		let new_p = original.translated(offset);
		// original + offset => (1 + 3.5, 2 + (-0.5)) = (4.5, 1.5)
		assert_eq!(new_p.x, 4.5);
		assert_eq!(new_p.y, 1.5);
		// original is unchanged by `translated()`, since it consumes and returns a new Point
		assert_eq!(original.x, 1.0);
		assert_eq!(original.y, 2.0);
	}

	#[test]
	fn test_point_translate_in_place() {
		let mut p = Point::new(2.0, 3.0);
		let offset = Point::new(-2.0, 1.0);
		p.translate(offset);
		// Now p should be updated => (0, 4)
		assert_eq!(p.x, 0.0);
		assert_eq!(p.y, 4.0);
	}
}
