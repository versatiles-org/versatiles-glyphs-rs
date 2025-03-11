use super::glyph::PbfGlyph;
use prost::{alloc, Message};

/// A collection of glyph information for a particular fontstack.
///
/// This struct captures metadata regarding a fontstack's name,
/// its supported glyph ID range, and the glyphs themselves.
#[derive(Clone, PartialEq, Message)]
pub struct Fontstack {
	/// The human-readable name of the fontstack.
	///
	/// Typically corresponds to a font family name or alias.
	#[prost(string, required, tag = "1")]
	pub name: alloc::string::String,

	/// A string describing the range of glyph IDs available
	/// in this fontstack, e.g., `"0-255"` or `"100-200"`.
	#[prost(string, required, tag = "2")]
	pub range: alloc::string::String,

	/// A list of [`Glyph`] structs describing individual glyph data,
	/// such as their bitmap, dimensions, offsets, and advance width.
	#[prost(message, repeated, tag = "3")]
	pub glyphs: alloc::vec::Vec<PbfGlyph>,
}

impl Fontstack {
	/// Creates a new [`Fontstack`] with the provided `name` and `range`,
	/// initializing an empty glyphs list.
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
		let glyph_a = PbfGlyph {
			id: 65,
			bitmap: Some(vec![1, 2, 3]),
			width: 12,
			height: 15,
			left: -1,
			top: 8,
			advance: 14,
		};
		let glyph_b = PbfGlyph {
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

		// Round-trip via protobuf encoding/decoding
		let encoded_data = fontstack.encode_to_vec();
		let decoded_fontstack = Fontstack::decode(&encoded_data[..]).unwrap();

		assert_eq!(
			format!("{decoded_fontstack:?}"),
			 "Fontstack { name: \"TestStack\", range: \"100-200\", glyphs: [PbfGlyph { id: 65, bitmap: Some([1, 2, 3]), width: 12, height: 15, left: -1, top: 8, advance: 14 }, PbfGlyph { id: 66, bitmap: None, width: 10, height: 11, left: 0, top: 5, advance: 12 }] }"
		);
	}
}
