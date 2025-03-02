use super::glyph::Glyph;
use prost::{alloc, Message};

/// Stores fontstack information and a list of faces.
#[derive(Clone, PartialEq, Message)]
pub struct Fontstack {
	#[prost(string, required, tag = "1")]
	pub name: alloc::string::String,
	#[prost(string, required, tag = "2")]
	pub range: alloc::string::String,
	#[prost(message, repeated, tag = "3")]
	pub glyphs: alloc::vec::Vec<Glyph>,
}

impl Fontstack {
	pub fn new(name: String, range: String) -> Self {
		Fontstack {
			name,
			range,
			glyphs: Vec::new(),
		}
	}
}
