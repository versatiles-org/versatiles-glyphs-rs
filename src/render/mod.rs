//! Rendering traits and implementations for generating SDF.
//!
//! # Overview
//! This module defines constants and submodules related to glyph rendering.
//!
//! # SDF coordinate model and the bbox rounding artifact
//!
//! Glyphs from the font are arbitrary cubic/quadratic curves in floating-point
//! EM units. To produce a [maplibre/mapbox-style SDF glyph](https://maplibre.org/),
//! the renderer flattens those curves to a fixed pixel grid (24px per EM after
//! [`renderer::Renderer::render_glyph`] scales them) and stores a signed-distance
//! bitmap with the metrics `width`, `height`, `left`, `top`, `advance`.
//!
//! Because every metric in the PBF is an integer, the float bbox of the
//! outline is converted as:
//!
//! ```text
//! left  = floor(min.x)         top  = ceil(max.y)
//! right = ceil(max.x)          bot  = floor(min.y)
//! ```
//!
//! `floor`/`ceil` (rather than `round`) guarantee the integer cell *contains*
//! the float bbox. The cost is a rounding artifact: the rings can sit anywhere
//! between 0 and 1 pixel inside each integer edge. Combined with the sub-pixel
//! shift used to compensate for `advance` rounding (see
//! [`renderer::Renderer::render_glyph`]), the actual glyph outline never lines
//! up exactly with pixel boundaries.
//!
//! # Why `BUFFER` is 3 even though the SDF radius is 8
//!
//! `renderer_precise` computes signed
//! distances out to `max_radius = 8.0` pixels. A faithful SDF would therefore
//! need 8 pixels of padding on every side so the gradient can decay to its
//! "fully outside" / "fully inside" plateau before being clipped.
//!
//! The maplibre/mapbox PBF format instead stores only `BUFFER` = 3 pixels of
//! padding. The bitmap actually written to disk is
//! `(width + 2·BUFFER) × (height + 2·BUFFER)`; everything beyond that
//! 3-pixel ring is dropped. Pixels in the buffer zone therefore hold partial
//! gradient values (distances 0–3 from the outline plus up to ~1 extra pixel
//! of slack from the floor/ceil rounding above). Pixels at distances 3–8
//! would be representable in the SDF model but simply do not fit in the file
//! format — hence "the buffer does not always fit". This is a deliberate
//! size-vs-quality tradeoff baked into the spec, not a bug in this renderer.
//!
//! `CUTOFF` = `0.25 * 256` is the SDF zero-crossing offset: the byte value
//! `192 = 256 - 64` corresponds to "exactly on the outline", with values below
//! falling off into the buffer and values above representing the interior.

/// Pixels of padding on every side of the glyph content area.
///
/// See the module-level docs for the relationship between this constant and
/// the SDF gradient radius (`max_radius` in `renderer_precise`).
const BUFFER: i32 = 3;

/// SDF zero-crossing offset, in the 0..=255 byte range used by the bitmap.
const CUTOFF: f64 = 0.25 * 256.0;

mod renderer;
mod renderer_dummy;
mod renderer_precise;
mod result;
mod ring_builder;
mod rtree_segments;

pub use renderer::Renderer;
pub use result::RenderResult;
