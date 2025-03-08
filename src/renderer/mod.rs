const BUFFER: i32 = 3;
const CUTOFF: f64 = 0.25 * 256.0;

mod render_glyph;
mod render_result;
#[cfg(test)]
mod renderer_dummy;
mod renderer_precise;
mod ring_builder;
mod rtree_segments;
mod traits;

pub use render_glyph::render_glyph;
pub use render_result::RenderResult;
#[cfg(test)]
pub use renderer_dummy::RendererDummy;
pub use renderer_precise::RendererPrecise;
pub use ring_builder::RingBuilder;
pub use traits::RendererTrait;
