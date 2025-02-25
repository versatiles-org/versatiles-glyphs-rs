use std::mem::swap;
use super::super::geometry::{Point, Ring, Rings};
use ttf_parser::OutlineBuilder;

pub struct RingBuilder {
	rings: Rings,
	ring: Ring,
}

impl RingBuilder {
	pub fn new() -> Self {
		RingBuilder {
			rings: Rings::new(),
			ring: Ring::new(),
		}
	}
	pub fn into_rings(mut self) -> Rings {
		self.save_ring();
		self.rings
	}
	fn save_ring(&mut self) {
		if !self.ring.is_empty() {
			self.ring.close();
			let mut ring = Ring::new();
			swap(&mut self.ring, &mut ring);
			self.rings.add_ring(ring);
		}
	}
}

impl OutlineBuilder for RingBuilder {
	fn move_to(&mut self, x: f32, y: f32) {
		self.save_ring();
		self.ring.add_point(Point::new(x, y));
	}

	fn line_to(&mut self, x: f32, y: f32) {
		self.ring.add_point(Point::new(x, y));
	}

	fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
		if self.ring.is_empty() {
			return;
		}
		self.ring.add_quadratic_bezier(
			*self.ring.last().unwrap(),
			Point::new(x1, y1),
			Point::new(x, y),
			0.3,
		);
	}

	fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
		if self.ring.is_empty() {
			return;
		}
		self.ring.add_cubic_bezier(
			*self.ring.last().unwrap(),
			Point::new(x1, y1),
			Point::new(x2, y2),
			Point::new(x, y),
			0.3,
		);
	}

	fn close(&mut self) {
		self.ring.close();
	}
}
