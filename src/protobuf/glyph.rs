use prost::{alloc, Message};

#[derive(Clone, PartialEq, Message)]
pub struct Glyph {
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

impl Glyph {
	pub fn empty(id: u32, advance: u32) -> Self {
		Glyph {
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
