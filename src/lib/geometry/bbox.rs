use super::Point;

// A bounding box for an R-tree
#[derive(Clone, Copy, Debug)]
pub struct BBox {
	pub min: Point,
	pub max: Point,
}

impl Default for BBox {
    fn default() -> Self {
        Self::new()
    }
}

impl BBox {
	pub fn new() -> Self {
		BBox {
			min: Point::new(f32::INFINITY, f32::INFINITY),
			max: Point::new(f32::NEG_INFINITY, f32::NEG_INFINITY),
		}
	}
	pub fn width(&self) -> f32 {
		(self.max.x - self.min.x).max(0.0)
	}
	pub fn height(&self) -> f32 {
		(self.max.y - self.min.y).max(0.0)
	}
	pub fn is_empty(&self) -> bool {
		self.max.x <= self.min.x || self.max.y <= self.min.y
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_new_bbox() {
		let bbox = BBox::new();
		assert_eq!(bbox.min.x, f32::INFINITY);
		assert_eq!(bbox.min.y, f32::INFINITY);
		assert_eq!(bbox.max.x, f32::NEG_INFINITY);
		assert_eq!(bbox.max.y, f32::NEG_INFINITY);
	}

	#[test]
	fn test_bbox_include_point() {
		let mut bbox = BBox::new();

		bbox.include_point(&Point::new(1.0, 2.0));
		assert_eq!(bbox.min.x, 1.0);
		assert_eq!(bbox.min.y, 2.0);
		assert_eq!(bbox.max.x, 1.0);
		assert_eq!(bbox.max.y, 2.0);

		// Include a point that expands the bounding box
		bbox.include_point(&Point::new(-1.0, 0.5));
		assert_eq!(bbox.min.x, -1.0);
		assert_eq!(bbox.min.y, 0.5);
		assert_eq!(bbox.max.x, 1.0);
		assert_eq!(bbox.max.y, 2.0);

		// Include a point that further expands the bounding box
		bbox.include_point(&Point::new(3.0, -2.0));
		assert_eq!(bbox.min.x, -1.0);
		assert_eq!(bbox.min.y, -2.0);
		assert_eq!(bbox.max.x, 3.0);
		assert_eq!(bbox.max.y, 2.0);
	}

	#[test]
	fn test_bbox_width_height() {
		let mut bbox = BBox::new();
		bbox.include_point(&Point::new(1.0, 1.0));
		bbox.include_point(&Point::new(2.0, 3.0));

		// min = (1.0, 1.0), max = (2.0, 3.0)
		assert_eq!(bbox.width(), 1.0);
		assert_eq!(bbox.height(), 2.0);
	}

	#[test]
	fn test_bbox_include_bbox() {
		let mut bbox1 = BBox::new();
		bbox1.include_point(&Point::new(1.0, 2.0));
		bbox1.include_point(&Point::new(-1.0, 0.5));

		let mut bbox2 = BBox::new();
		bbox2.include_point(&Point::new(3.0, -2.0));
		bbox2.include_point(&Point::new(2.0, 3.0));

		// bbox1 covers: min(-1.0, 0.5), max(1.0, 2.0)
		// bbox2 covers: min(2.0, -2.0), max(3.0, 3.0)

		// Include bbox2 into bbox1
		bbox1.include_bbox(&bbox2);

		// Now bbox1 should cover the combined area:
		// min.x = -1.0, min.y = -2.0
		// max.x = 3.0,  max.y = 3.0
		assert_eq!(bbox1.min.x, -1.0);
		assert_eq!(bbox1.min.y, -2.0);
		assert_eq!(bbox1.max.x, 3.0);
		assert_eq!(bbox1.max.y, 3.0);
	}

	#[test]
	fn test_bbox_round() {
		let mut bbox = BBox {
			min: Point::new(1.4, 2.6),
			max: Point::new(3.7, -1.2),
		};
		bbox.round();
		// Rounding each component individually:
		// min.x = 1.4  -> 1.0
		// min.y = 2.6  -> 3.0
		// max.x = 3.7  -> 4.0
		// max.y = -1.2 -> -1.0
		assert_eq!(bbox.min.x, 1.0);
		assert_eq!(bbox.min.y, 3.0);
		assert_eq!(bbox.max.x, 4.0);
		assert_eq!(bbox.max.y, -1.0);
	}

	#[test]
	fn test_new_bbox_is_empty() {
		let mut bbox = BBox::new();
		assert!(bbox.is_empty());

		bbox.include_point(&Point::new(2.0, 3.0));
		assert!(bbox.is_empty());

		bbox.include_point(&Point::new(2.0, 5.0));
		assert!(bbox.is_empty());

		bbox.include_point(&Point::new(1.0, 5.0));
		assert!(!bbox.is_empty());
	}
}
