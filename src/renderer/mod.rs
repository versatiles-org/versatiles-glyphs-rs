const BUFFER: i32 = 3;
const CUTOFF: f64 = 0.25 * 256.0;

mod glyph;
#[cfg(test)]
mod render_dummy;
mod render_precise;
mod rtree_segments;
mod traits;

pub use glyph::SdfGlyph;
#[cfg(test)]
pub use render_dummy::RendererDummy;
pub use render_precise::RendererPrecise;
pub use traits::RendererTrait;
