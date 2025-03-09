//! Geometric primitives and operations used for glyph outlines.
//!
//! This module provides fundamental 2D geometric primitives and structures, including:
//!
//! - **[`Point`]:** A 2D coordinate with methods for arithmetic and transformations.
//! - **[`Ring`]:** A sequence of [`Point`]s that can form a closed loop, used for polygonal outlines.
//! - **[`Rings`]:** A collection of multiple [`Ring`] objects, representing complex or multi-part shapes.
//! - **[`BBox`]:** An axis-aligned bounding box that expands to include additional points or boxes.
//! - **[`Segment`]:** A line segment defined by two [`Point`] references, with operations like projection.
//!
//! These types are commonly used throughout the glyph rendering pipeline for outline calculations,
//! geometric transformations, intersection checks, and more.

mod bbox;
mod point;
mod ring;
mod rings;
mod segment;

pub use bbox::BBox;
pub use point::Point;
pub use ring::Ring;
pub use rings::Rings;
pub use segment::Segment;
