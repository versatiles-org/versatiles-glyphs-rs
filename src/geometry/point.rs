#![allow(dead_code)]

#[derive(Clone, Copy, Debug, PartialEq)]
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

	pub fn scale(&mut self, scale: f32) {
		self.x *= scale;
		self.y *= scale;
	}

	pub fn as_tuple(&self) -> (f32, f32) {
		(self.x, self.y)
	}
}

impl From<(f32, f32)> for Point {
	fn from(t: (f32, f32)) -> Self {
		Point::new(t.0, t.1)
	}
}

impl From<(i32, i32)> for Point {
	fn from(t: (i32, i32)) -> Self {
		Point::new(t.0 as f32, t.1 as f32)
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
		let new_p = original.translated(offset);
		assert_eq!(new_p.as_tuple(), (4.5, 1.5));
		assert_eq!(original.as_tuple(), (1.0, 2.0));
	}

	#[test]
	fn test_point_translate_in_place() {
		let mut p = Point::new(2.0, 3.0);
		let offset = Point::new(-2.0, 1.0);
		p.translate(offset);
		assert_eq!(p.as_tuple(), (0.0, 4.0));
	}

	#[test]
	fn test_point_scale() {
		let mut p = Point::new(2.0, 3.0);
		p.scale(4.0);
		assert_eq!(p.as_tuple(), (8.0, 12.0));
	}
}
