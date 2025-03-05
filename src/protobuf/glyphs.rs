use super::fontstack::Fontstack;
use prost::{alloc, Message};

#[derive(Clone, PartialEq, Message)]
pub struct Glyphs {
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
