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
