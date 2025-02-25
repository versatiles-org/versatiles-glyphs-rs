use super::types::{Point, Ring, Rings, Segment};

fn approx_distance_squared(a: Point, b: Point) -> f64 {
	let dx = b.x - a.x;
	let dy = b.y - a.y;
	dx * dx + dy * dy
}

/// Flatten a quadratic Bézier defined by [start, ctrl, end] into a sequence of points.
/// `tolerance_sq` is a threshold for subdivision.
fn flatten_conic(
	start: Point,
	ctrl: Point,
	end: Point,
	tolerance_sq: f64,
	output: &mut Vec<Point>,
) {
	// Evaluate midpoints
	let mid_1 = Point::new((start.x + ctrl.x) * 0.5, (start.y + ctrl.y) * 0.5);
	let mid_2 = Point::new((ctrl.x + end.x) * 0.5, (ctrl.y + end.y) * 0.5);
	let mid = Point::new((mid_1.x + mid_2.x) * 0.5, (mid_1.y + mid_2.y) * 0.5);

	// We check if the curve is "flat enough"
	let dx = start.x + end.x - ctrl.x * 2.0;
	let dy = start.y + end.y - ctrl.y * 2.0;
	let dist_sq = dx * dx + dy * dy;

	if dist_sq <= tolerance_sq {
		// It's flat enough, just line to the end
		output.push(end);
	} else {
		// Subdivide
		flatten_conic(start, mid_1, mid, tolerance_sq, output);
		flatten_conic(mid, mid_2, end, tolerance_sq, output);
	}
}

/// Flatten a cubic Bézier [start, c1, c2, end].
fn flatten_cubic(
	start: Point,
	c1: Point,
	c2: Point,
	end: Point,
	tolerance_sq: f64,
	output: &mut Vec<Point>,
) {
	// Using De Casteljau or similar approach.
	// Compute midpoints
	let p01 = midpoint(start, c1);
	let p12 = midpoint(c1, c2);
	let p23 = midpoint(c2, end);
	let p012 = midpoint(p01, p12);
	let p123 = midpoint(p12, p23);
	let mid = midpoint(p012, p123);

	// Check "flatness" by approximating the distance from midpoints
	let dx = (c2.x + c1.x) - (start.x + end.x);
	let dy = (c2.y + c1.y) - (start.y + end.y);
	let dist_sq = dx * dx + dy * dy;

	if dist_sq <= tolerance_sq {
		// Flat enough
		output.push(end);
	} else {
		// Subdivide
		flatten_cubic(start, p01, p012, mid, tolerance_sq, output);
		flatten_cubic(mid, p123, p23, end, tolerance_sq, output);
	}
}

fn midpoint(a: Point, b: Point) -> Point {
	Point::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5)
}

// --------------------------------------------------------------------
// 3) The "User" struct & Decompose Functions
//    The C++ code uses FT_Outline_Decompose callbacks to build rings.
//    We'll replicate them in pure Rust style.
//    Typically, you'd call these from your FreeType outline iteration
//    or from ttf-parser's outline builder interface.
// --------------------------------------------------------------------

struct User {
	pub rings: Rings,
	pub ring: Ring,
}

/// Ensures the ring is closed by repeating the first point if needed.
fn close_ring(ring: &mut Ring) {
	if ring.is_empty() {
		return;
	}
	let first = ring[0];
	let last = ring[ring.len() - 1];
	if (first.x - last.x).abs() > f64::EPSILON || (first.y - last.y).abs() > f64::EPSILON {
		ring.push(first);
	}
}

/// Simulates the MoveTo callback
fn move_to(x: f64, y: f64, user: &mut User) {
	if !user.ring.is_empty() {
		// close previous ring
		close_ring(&mut user.ring);
		user.rings.push(user.ring.clone());
		user.ring.clear();
	}
	user.ring.push(Point::new(x, y));
}

/// LineTo
fn line_to(x: f64, y: f64, user: &mut User) {
	user.ring.push(Point::new(x, y));
}

/// ConicTo (Quadratic Bézier)
fn conic_to(ctrl_x: f64, ctrl_y: f64, to_x: f64, to_y: f64, user: &mut User) {
	if user.ring.is_empty() {
		return;
	}
	let prev = *user.ring.last().unwrap();
	// Remove last repeated point
	user.ring.pop();

	// Flatten the curve
	let mut flattened = Vec::new();
	let tolerance_sq = 0.3; // tweak as needed
	flatten_conic(
		prev,
		Point::new(ctrl_x, ctrl_y),
		Point::new(to_x, to_y),
		tolerance_sq,
		&mut flattened,
	);

	// Re-insert
	user.ring.push(prev);
	user.ring.extend_from_slice(&flattened);
}

/// CubicTo
fn cubic_to(c1x: f64, c1y: f64, c2x: f64, c2y: f64, to_x: f64, to_y: f64, user: &mut User) {
	if user.ring.is_empty() {
		return;
	}
	let prev = *user.ring.last().unwrap();
	user.ring.pop();

	let mut flattened = Vec::new();
	let tolerance_sq = 0.3; // tweak
	flatten_cubic(
		prev,
		Point::new(c1x, c1y),
		Point::new(c2x, c2y),
		Point::new(to_x, to_y),
		tolerance_sq,
		&mut flattened,
	);

	user.ring.push(prev);
	user.ring.extend_from_slice(&flattened);
}

// --------------------------------------------------------------------
// 4) Winding-based "point in polygon" test
// --------------------------------------------------------------------

fn is_left(p0: Point, p1: Point, p2: Point) -> i32 {
	let val = (p1.x - p0.x) * (p2.y - p0.y) - (p2.x - p0.x) * (p1.y - p0.y);
	if val > 0.0 {
		1
	} else if val < 0.0 {
		-1
	} else {
		0
	}
}

fn poly_contains_point(rings: &Rings, pt: Point) -> bool {
	let mut winding_number = 0;
	for ring in rings {
		if ring.len() < 2 {
			continue;
		}
		let mut p1 = ring[0];
		for p2 in ring.iter().skip(1) {
			if p1.y <= pt.y {
				if p2.y > pt.y {
					if is_left(p1, *p2, pt) > 0 {
						winding_number += 1;
					}
				}
			} else {
				if p2.y <= pt.y {
					if is_left(p1, *p2, pt) < 0 {
						winding_number -= 1;
					}
				}
			}
			p1 = *p2;
		}
	}
	winding_number != 0
}

fn squared_distance(a: Point, b: Point) -> f64 {
	let dx = b.x - a.x;
	let dy = b.y - a.y;
	dx * dx + dy * dy
}

fn project_point_on_segment(p: Point, seg: Segment) -> Point {
	let v = seg.start;
	let w = seg.end;
	let l2 = squared_distance(v, w);
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

pub fn squared_distance_to_segment(p: Point, seg: Segment) -> f64 {
	let proj = project_point_on_segment(p, seg);
	squared_distance(p, proj)
}
