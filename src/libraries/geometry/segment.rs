use super::Point;

// A line segment is just two Points
#[derive(Clone, Copy, Debug)]
pub struct Segment {
	pub start: Point,
	pub end: Point,
}

impl Segment {
	pub fn new(start: Point, end: Point) -> Self {
		Segment { start, end }
	}

	pub fn project_point_on(&self, p: &Point) -> Point {
		let v = self.start;
		let w = self.end;
		let l2 = v.squared_distance_to(&w);
		if l2 == 0.0 {
			return v;
		}
		let t = ((p.x - v.x) * (w.x - v.x) + (p.y - v.y) * (w.y - v.y)) / l2;
		if t < 0.0 {
			return v;
		} else if t > 1.0 {
			return w;
		}
		Point::new(v.x + t * (w.x - v.x), v.y + t * (w.y - v.y))
	}

	pub fn squared_distance_to_point(&self, p: &Point) -> f32 {
		let proj = self.project_point_on(p);
		p.squared_distance_to(&proj)
	}
}
