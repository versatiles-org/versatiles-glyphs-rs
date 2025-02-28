use crate::sdf::SdfGlyph;
use prost::{alloc, Message};

#[derive(Clone, PartialEq, Message)]
pub struct PbfGlyph {
	#[prost(uint32, required, tag = "1")]
	pub id: u32,
	/// A signed distance field of the glyph with a border of 3 pixels.
	#[prost(bytes = "vec", optional, tag = "2")]
	pub bitmap: Option<alloc::vec::Vec<u8>>,
	/// Glyph metrics.
	#[prost(uint32, required, tag = "3")]
	pub width: u32,
	#[prost(uint32, required, tag = "4")]
	pub height: u32,
	#[prost(sint32, required, tag = "5")]
	pub left: i32,
	#[prost(sint32, required, tag = "6")]
	pub top: i32,
	#[prost(uint32, required, tag = "7")]
	pub advance: u32,
}

impl PbfGlyph {
	pub fn from_sdf(sdf: SdfGlyph, id: u32, advance: u32) -> Self {
		PbfGlyph {
			id,
			bitmap: Some(sdf.bitmap),
			width: sdf.width - 6,
			height: sdf.height - 6,
			left: sdf.left,
			top: sdf.top,
			advance,
		}
	}
	pub fn empty(id: u32, advance: u32) -> Self {
		PbfGlyph {
			id,
			bitmap: None,
			width: 0,
			height: 0,
			left: 0,
			top: 0,
			advance,
		}
	}
}

/// Stores fontstack information and a list of faces.
#[derive(Clone, PartialEq, Message)]
pub struct PbfFontstack {
	#[prost(string, required, tag = "1")]
	pub name: alloc::string::String,
	#[prost(string, required, tag = "2")]
	pub range: alloc::string::String,
	#[prost(message, repeated, tag = "3")]
	pub glyphs: alloc::vec::Vec<PbfGlyph>,
}

#[derive(Clone, PartialEq, Message)]
pub struct PbfGlyphs {
	#[prost(message, repeated, tag = "1")]
	pub stacks: alloc::vec::Vec<PbfFontstack>,
}
