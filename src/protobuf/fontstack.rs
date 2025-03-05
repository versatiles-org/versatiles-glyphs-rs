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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_fontstack_new() {
		let fontstack = Fontstack::new("TestFont".to_string(), "0-255".to_string());
		assert_eq!(
			format!("{fontstack:?}"),
			"Fontstack { name: \"TestFont\", range: \"0-255\", glyphs: [] }"
		);
	}

	#[test]
	fn test_fontstack_serialization_round_trip() {
		let mut fontstack = Fontstack::new("TestStack".to_string(), "100-200".to_string());

		// Create a few glyphs
		let glyph_a = Glyph {
			id: 65,
			bitmap: Some(vec![1, 2, 3]),
			width: 12,
			height: 15,
			left: -1,
			top: 8,
			advance: 14,
		};
		let glyph_b = Glyph {
			id: 66,
			bitmap: None,
			width: 10,
			height: 11,
			left: 0,
			top: 5,
			advance: 12,
		};
		fontstack.glyphs.push(glyph_a.clone());
		fontstack.glyphs.push(glyph_b.clone());

		let encoded_data = fontstack.encode_to_vec();
		let decoded_fontstack = Fontstack::decode(&encoded_data[..]).unwrap();

		assert_eq!(format!("{decoded_fontstack:?}"), "Fontstack { name: \"TestStack\", range: \"100-200\", glyphs: [Glyph { id: 65, bitmap: Some([1, 2, 3]), width: 12, height: 15, left: -1, top: 8, advance: 14 }, Glyph { id: 66, bitmap: None, width: 10, height: 11, left: 0, top: 5, advance: 12 }] }");
	}
}
