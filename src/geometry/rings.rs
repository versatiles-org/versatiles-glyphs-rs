//! A collection of multiple [`Ring`] instances, which can represent complex shapes
//! or multi-polygon outlines.
//!
//! Each [`Ring`] may form a closed loop, such as a polygon boundary. Together,
//! they can compose regions with holes, overlapping areas, etc. This module
//! also provides convenient methods for bounding box calculation, transformations,
//! and point-in-polygon tests.

use super::{BBox, Point, Ring, Segment};

/// A wrapper around multiple [`Ring`]s, enabling operations over
/// all rings simultaneously (e.g., bounding box calculation, translation).
#[derive(Clone, Debug, PartialEq)]
pub struct Rings {
	/// The individual [`Ring`]s that compose this collection.
	pub rings: Vec<Ring>,
}

impl Default for Rings {
	fn default() -> Self {
		Self::new()
	}
}

#[allow(dead_code)]
impl Rings {
	/// Creates an empty [`Rings`] collection with no [`Ring`]s.
	pub fn new() -> Self {
		Rings { rings: Vec::new() }
	}

	/// Returns the number of [`Ring`]s in this collection.
	pub fn len(&self) -> usize {
		self.rings.len()
	}

	/// Returns `true` if there are no [`Ring`]s in this collection.
	pub fn is_empty(&self) -> bool {
		self.rings.is_empty()
	}

	/// Adds a [`Ring`] to this collection.
	pub fn add_ring(&mut self, ring: Ring) {
		self.rings.push(ring);
	}

	/// Computes the [bounding box](BBox) that encloses every [`Ring`] in this collection.
	///
	/// This is done by merging all ring-level bounding boxes.
	pub fn get_bbox(&self) -> BBox {
		let mut bbox = BBox::new();
		for ring in &self.rings {
			bbox.include_bbox(&ring.get_bbox());
		}
		bbox
	}

	/// Translates all points in every [`Ring`] by the given offset.
	pub fn translate(&mut self, offset: &Point) {
		for ring in &mut self.rings {
			ring.translate(offset);
		}
	}

	/// Scales all points in every [`Ring`] by the given factor.
	pub fn scale(&mut self, scale: f64) {
		for ring in &mut self.rings {
			ring.scale(scale);
		}
	}

	/// Returns all [`Segment`]s from all [`Ring`]s in this collection.
	///
	/// Consecutive points in each ring form a segment, and the rings are processed in order.
	pub fn get_segments(&self) -> Vec<Segment> {
		self
			.rings
			.iter()
			.flat_map(|ring| ring.get_segments())
			.collect()
	}

	/// Determines whether the specified `pt` lies inside the area formed by any of
	/// the [`Ring`]s in this collection, based on winding number logic.
	///
	/// If at least one ring encloses the point (non-zero winding),
	/// this method returns `true`.
	pub fn contains_point(&self, pt: &Point) -> bool {
		let mut winding_number = 0;
		for ring in &self.rings {
			winding_number += ring.winding_number(pt);
		}
		winding_number != 0
	}
}

impl<T> From<Vec<T>> for Rings
where
	Ring: From<T>,
{
	/// Creates a new [`Rings`] object from a vector of items that
	/// can be converted into [`Ring`].
	///
	/// For example:
	/// ```
	/// use versatiles_glyphs::geometry::Rings;
	/// // Each sub-vector can become a Ring,
	/// // which in turn is constructed from Points or tuples.
	/// let all_rings = vec![
	///     vec![(0.0, 0.0), (1.0, 0.0)],
	///     vec![(2.0, 2.0), (3.0, 2.0)],
	/// ];
	/// let rings = Rings::from(all_rings);
	/// assert_eq!(rings.len(), 2);
	/// ```
	fn from(rings: Vec<T>) -> Self {
		Rings {
			rings: rings.into_iter().map(Ring::from).collect(),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_rings_new() {
		let rings = Rings::new();
		assert_eq!(rings.rings.len(), 0);
	}

	#[test]
	fn test_add_ring() {
		let mut rings = Rings::new();
		let ring = Ring::new();
		rings.add_ring(ring);
		assert_eq!(rings.rings.len(), 1);
	}

	#[test]
	fn test_get_bbox_multiple_rings() {
		let mut rings = Rings::new();
		// Ring 1: bounding box from (0,0) to (2,2)
		let mut ring1 = Ring::new();
		ring1.add_point(Point::new(0.0, 0.0));
		ring1.add_point(Point::new(2.0, 2.0));
		ring1.close();

		// Ring 2: bounding box from (3, -1) to (5, 1)
		let mut ring2 = Ring::new();
		ring2.add_point(Point::new(3.0, -1.0));
		ring2.add_point(Point::new(5.0, 1.0));
		ring2.close();

		rings.add_ring(ring1);
		rings.add_ring(ring2);

		let bbox = rings.get_bbox();
		// The combined bounding box should be (0, -1) to (5, 2)
		assert_eq!(bbox.min.as_tuple(), (0.0, -1.0));
		assert_eq!(bbox.max.as_tuple(), (5.0, 2.0));
	}

	#[test]
	fn test_translate() {
		let mut rings = Rings::new();
		let mut ring = Ring::new();
		ring.add_point(Point::new(0.0, 0.0));
		ring.add_point(Point::new(1.0, 1.0));
		ring.close();

		rings.add_ring(ring);
		rings.translate(&Point::new(2.0, 3.0));

		let translated_ring = &rings.rings[0];
		assert_eq!(translated_ring.points[0].as_tuple(), (2.0, 3.0));
		assert_eq!(translated_ring.points[1].as_tuple(), (3.0, 4.0));
		assert_eq!(translated_ring.points[2].as_tuple(), (2.0, 3.0));
	}

	#[test]
	fn test_scale() {
		let mut rings = Rings::new();
		let mut ring = Ring::new();
		ring.add_point(Point::new(0.0, 1.0));
		ring.add_point(Point::new(2.0, 3.0));
		ring.close();

		rings.add_ring(ring);
		rings.scale(2.0);

		let scaled_ring = &rings.rings[0];
		assert_eq!(scaled_ring.points[0].as_tuple(), (0.0, 2.0));
		assert_eq!(scaled_ring.points[1].as_tuple(), (4.0, 6.0));
		assert_eq!(scaled_ring.points[2].as_tuple(), (0.0, 2.0));
	}

	#[test]
	fn test_get_segments() {
		let mut rings = Rings::new();

		// First ring has 3 points => 2 segments
		let mut ring1 = Ring::new();
		ring1.add_point(Point::new(0.0, 0.0));
		ring1.add_point(Point::new(1.0, 0.0));
		ring1.add_point(Point::new(1.0, 1.0));

		// Second ring has 4 points => 3 segments
		let mut ring2 = Ring::new();
		ring2.add_point(Point::new(2.0, 2.0));
		ring2.add_point(Point::new(3.0, 2.0));
		ring2.add_point(Point::new(3.0, 3.0));
		ring2.add_point(Point::new(2.0, 3.0));

		rings.add_ring(ring1);
		rings.add_ring(ring2);

		let segments = rings.get_segments();
		// total segments = 2 + 3 = 5
		assert_eq!(segments.len(), 5);

		// First ring's first segment: (0,0)->(1,0)
		assert_eq!(segments[0].start.as_tuple(), (0.0, 0.0));
		assert_eq!(segments[0].end.as_tuple(), (1.0, 0.0));

		// Second ring's last segment: (3,3)->(2,3)
		let last_seg = &segments[4];
		assert_eq!(last_seg.start.as_tuple(), (3.0, 3.0));
		assert_eq!(last_seg.end.as_tuple(), (2.0, 3.0));
	}

	#[test]
	fn test_contains_point() {
		let mut rings = Rings::new();

		// A simple square ring around the origin
		let mut ring1 = Ring::new();
		ring1.add_point(Point::new(0.0, 0.0));
		ring1.add_point(Point::new(10.0, 0.0));
		ring1.add_point(Point::new(10.0, 10.0));
		ring1.add_point(Point::new(0.0, 10.0));
		ring1.close();

		rings.add_ring(ring1);

		let inside = Point::new(5.0, 5.0);
		let outside = Point::new(11.0, 5.0);

		assert!(
			rings.contains_point(&inside),
			"Inside point should be contained."
		);
		assert!(
			!rings.contains_point(&outside),
			"Outside point should not be contained."
		);
	}

	#[test]
	fn test_contains_point_multiple_rings() {
		let mut rings = Rings::new();

		// Ring 1: a small square from (0,0) to (2,2)
		let mut ring1 = Ring::new();
		ring1.add_point(Point::new(0.0, 0.0));
		ring1.add_point(Point::new(2.0, 0.0));
		ring1.add_point(Point::new(2.0, 2.0));
		ring1.add_point(Point::new(0.0, 2.0));
		ring1.close();

		// Ring 2: a second square from (3,3) to (5,5)
		let mut ring2 = Ring::new();
		ring2.add_point(Point::new(3.0, 3.0));
		ring2.add_point(Point::new(5.0, 3.0));
		ring2.add_point(Point::new(5.0, 5.0));
		ring2.add_point(Point::new(3.0, 5.0));
		ring2.close();

		rings.add_ring(ring1);
		rings.add_ring(ring2);

		// Inside ring1
		let inside_ring1 = Point::new(1.0, 1.0);
		assert!(rings.contains_point(&inside_ring1));

		// Inside ring2
		let inside_ring2 = Point::new(4.0, 4.0);
		assert!(rings.contains_point(&inside_ring2));

		// Outside both
		let outside_all = Point::new(10.0, 10.0);
		assert!(!rings.contains_point(&outside_all));
	}
}
