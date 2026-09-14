use super::glyph::PbfGlyph;
use prost::{alloc, Message};

/// A collection of glyph information for a particular fontstack.
///
/// The schema defines a `name` and a `range`, but neither is written: clients
/// (MapLibre GL JS and MapLibre Native) only read the glyphs, and omitting both
/// keeps every PBF independent of the font it belongs to. This also means an empty
/// range encodes to just two bytes (`0a 00`). Both fields are still decoded if present.
#[derive(Clone, PartialEq, Message)]
pub struct Fontstack {
	/// The human-readable name of the fontstack. Not written.
	#[prost(string, optional, tag = "1")]
	pub name: Option<alloc::string::String>,

	/// A string describing the range of glyph IDs available
	/// in this fontstack, e.g., `"0-255"`. Not written.
	#[prost(string, optional, tag = "2")]
	pub range: Option<alloc::string::String>,

	/// A list of [`PbfGlyph`] structs describing individual glyph data,
	/// such as their bitmap, dimensions, offsets, and advance width.
	#[prost(message, repeated, tag = "3")]
	pub glyphs: alloc::vec::Vec<PbfGlyph>,
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_fontstack_default_encodes_empty() {
		assert_eq!(Fontstack::default().encode_to_vec(), Vec::<u8>::new());
	}

	#[test]
	fn test_fontstack_decodes_name_and_range() {
		// A fontstack written by older versions: name "TestFont", range "0-255".
		let data = b"\x0a\x08TestFont\x12\x050-255";
		let fontstack = Fontstack::decode(&data[..]).unwrap();
		assert_eq!(
			format!("{fontstack:?}"),
			"Fontstack { name: Some(\"TestFont\"), range: Some(\"0-255\"), glyphs: [] }"
		);
	}

	#[test]
	fn test_fontstack_serialization_round_trip() {
		let mut fontstack = Fontstack::default();

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
			 "Fontstack { name: None, range: None, glyphs: [PbfGlyph { id: 65, bitmap: Some([1, 2, 3]), width: 12, height: 15, left: -1, top: 8, advance: 14 }, PbfGlyph { id: 66, bitmap: None, width: 10, height: 11, left: 0, top: 5, advance: 12 }] }"
		);
	}
}
