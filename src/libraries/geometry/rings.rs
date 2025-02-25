use super::{BBox, Point, Ring, Segment};

pub struct Rings {
	rings: Vec<Ring>,
}

impl Rings {
	pub fn new() -> Self {
		Rings { rings: Vec::new() }
	}
	pub fn add_ring(&mut self, ring: Ring) {
		self.rings.push(ring);
	}

	pub fn get_bbox(&self) -> BBox {
		let mut bbox = BBox::new();
		for ring in &self.rings {
			bbox.include_bbox(&ring.get_bbox());
		}
		bbox
	}

	pub fn translate(&mut self, offset: Point) {
		for ring in &mut self.rings {
			ring.translate(offset);
		}
	}

	pub fn get_segments(&self) -> Vec<Segment> {
		self
			.rings
			.iter()
			.flat_map(|ring| ring.get_segments())
			.collect()
	}

	pub fn contains_point(&self, pt: &Point) -> bool {
		let mut winding_number = 0;
		for ring in self.rings.iter() {
			winding_number += ring.winding_number(pt);
		}
		winding_number != 0
	}
}
