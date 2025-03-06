const BUFFER: i32 = 3;
const CUTOFF: f64 = 0.25 * 256.0;

mod glyph;
mod render_precise;
mod rtree_segments;
mod traits;

pub use glyph::SdfGlyph;
pub use render_precise::RendererPrecise;
pub use traits::RendererTrait;
