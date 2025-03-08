use super::fontstack::Fontstack;
use prost::{alloc, Message};

/// A collection of one or more [`Fontstack`] instances,
/// each containing glyph data for a particular font or range.
///
/// This top-level message is often used to represent an entire
/// set of glyphs for multiple fonts in a single protobuf structure.
#[derive(Clone, PartialEq, Message)]
pub struct Glyphs {
	/// A list of [`Fontstack`] objects, where each entry
	/// corresponds to a unique font name or ID range.
	#[prost(message, repeated, tag = "1")]
	pub stacks: alloc::vec::Vec<Fontstack>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_glyphs_new() {
		let glyphs = Glyphs {
			stacks: vec![Fontstack::new("TestFont".to_string(), "0-255".to_string())],
		};

		let encoded_data = glyphs.encode_to_vec();
		let decoded_glyphs = Glyphs::decode(&encoded_data[..]).unwrap();

		assert_eq!(
			format!("{decoded_glyphs:?}"),
			"Glyphs { stacks: [Fontstack { name: \"TestFont\", range: \"0-255\", glyphs: [] }] }"
		);
	}
}
