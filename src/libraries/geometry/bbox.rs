use super::Point;

// A bounding box for an R-tree
#[derive(Clone, Copy, Debug)]
pub struct BBox {
	pub min: Point,
	pub max: Point,
}

impl BBox {
	pub fn new() -> Self {
		BBox {
			min: Point::new(f32::INFINITY, f32::INFINITY),
			max: Point::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
		}
	}
	pub fn width(&self) -> f32 {
		self.max.x - self.min.x
	}
	pub fn height(&self) -> f32 {
		self.max.y - self.min.y
	}
	pub fn include_point(&mut self, point: &Point) {
		self.min.x = self.min.x.min(point.x);
		self.min.y = self.min.y.min(point.y);
		self.max.x = self.max.x.max(point.x);
		self.max.y = self.max.y.max(point.y);
	}
	pub fn include_bbox(&mut self, other: &BBox) {
		self.min.x = self.min.x.min(other.min.x);
		self.min.y = self.min.y.min(other.min.y);
		self.max.x = self.max.x.max(other.max.x);
		self.max.y = self.max.y.max(other.max.y);
	}
	pub fn round(&mut self) {
		self.min.x = self.min.x.round();
		self.min.y = self.min.y.round();
		self.max.x = self.max.x.round();
		self.max.y = self.max.y.round();
	}
}
