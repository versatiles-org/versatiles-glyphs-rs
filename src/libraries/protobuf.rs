mod protobuf {
	include!(concat!(env!("OUT_DIR"), "/llmr.glyphs.rs"));
}

pub use protobuf::Fontstack as PbfFontstack;
pub use protobuf::Glyph as PbfGlyph;
