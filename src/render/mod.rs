//! Rendering traits and implementations for generating SDF.
//!
//! # Overview
//! This module defines constants and submodules related to glyph rendering

const BUFFER: i32 = 3;
const CUTOFF: f64 = 0.25 * 256.0;

mod renderer;
mod renderer_dummy;
mod renderer_precise;
mod result;
mod ring_builder;
mod rtree_segments;

pub use renderer::Renderer;
pub use result::RenderResult;
