//! Provides an interface for rendering glyph outlines into bitmap data.
//!
//! # Overview
//! This module defines constants and submodules related to glyph rendering

const BUFFER: i32 = 3;
const CUTOFF: f64 = 0.25 * 256.0;

mod render_glyph;
#[cfg(test)]
mod renderer_dummy;
mod renderer_precise;
mod result;
mod ring_builder;
mod rtree_segments;
mod traits;

pub use render_glyph::render_glyph;
#[cfg(test)]
pub use renderer_dummy::RendererDummy;
pub use renderer_precise::RendererPrecise;
pub use result::RenderResult;
pub use ring_builder::RingBuilder;
pub use traits::RendererTrait;
