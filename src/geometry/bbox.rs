use super::Point;

/// A bounding box that can expand to include points and merge with other bounding boxes.
/// Primarily used in spatial data structures (e.g., an R-tree) to track minimum and maximum
/// coordinates.
#[derive(Clone, Debug)]
pub struct BBox {
	/// The minimum (x, y) coordinates in this bounding box.
	pub min: Point,
	/// The maximum (x, y) coordinates in this bounding box.
	pub max: Point,
}

impl Default for BBox {
	fn default() -> Self {
		Self::new()
	}
}

#[allow(dead_code)]
impl BBox {
	/// Creates a new, "empty" bounding box with `min` set to `[∞, ∞]`
	/// and `max` set to `[-∞, -∞]`.
	///
	/// Such a box will expand to include any point or other bounding box
	/// added via [`self.include_point`] or [`self.include_bbox`].
	pub fn new() -> Self {
		BBox {
			min: Point::new(f64::INFINITY, f64::INFINITY),
			max: Point::new(f64::NEG_INFINITY, f64::NEG_INFINITY),
		}
	}

	/// Returns the width (`max.x - min.x`) of this bounding box,
	/// ensuring it is non-negative.
	pub fn width(&self) -> f64 {
		(self.max.x - self.min.x).max(0.0)
	}

	/// Returns the height (`max.y - min.y`) of this bounding box,
	/// ensuring it is non-negative.
	pub fn height(&self) -> f64 {
		(self.max.y - self.min.y).max(0.0)
	}

	/// Checks if this bounding box is empty. A box is considered empty
	/// if `max.x <= min.x` or `max.y <= min.y`.
	///
	/// This is typically true only for a newly-created bounding box before
	/// any points expand its bounds, or if points are collinear in an
	/// unexpected way.
	pub fn is_empty(&self) -> bool {
		self.max.x <= self.min.x || self.max.y <= self.min.y
	}

	/// Expands this bounding box to include the given [`Point`].
	///
	/// If the point is outside the existing bounds, the `min` or `max`
	/// fields will be updated accordingly.
	pub fn include_point(&mut self, point: &Point) {
		self.min.x = self.min.x.min(point.x);
		self.min.y = self.min.y.min(point.y);
		self.max.x = self.max.x.max(point.x);
		self.max.y = self.max.y.max(point.y);
	}

	/// Expands this bounding box to include the extents of another [`BBox`].
	///
	/// If the other box extends beyond the current `min` or `max`,
	/// they will be updated accordingly.
	pub fn include_bbox(&mut self, other: &BBox) {
		self.min.x = self.min.x.min(other.min.x);
		self.min.y = self.min.y.min(other.min.y);
		self.max.x = self.max.x.max(other.max.x);
		self.max.y = self.max.y.max(other.max.y);
	}

	/// Rounds all coordinates (`min` and `max`) to the nearest integer value.
	///
	/// This is particularly useful when converting fractional coordinates
	/// to discrete pixel coordinates in rendering or raster-based scenarios.
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
		assert_eq!(bbox.min.x, f64::INFINITY);
		assert_eq!(bbox.min.y, f64::INFINITY);
		assert_eq!(bbox.max.x, f64::NEG_INFINITY);
		assert_eq!(bbox.max.y, f64::NEG_INFINITY);
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
		// Rounding each component:
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
