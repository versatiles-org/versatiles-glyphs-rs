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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_empty_glyph() {
		let glyph_id = 42;
		let advance = 100;
		let original_glyph = Glyph::empty(glyph_id, advance);

		let encoded = original_glyph.encode_to_vec();
		let decoded_glyph = Glyph::decode(&encoded[..]).unwrap();

		assert_eq!(
			format!("{decoded_glyph:?}"),
			"Glyph { id: 42, bitmap: None, width: 0, height: 0, left: 0, top: 0, advance: 100 }"
		);
	}

	#[test]
	fn test_serialization_round_trip() {
		let original_glyph = Glyph {
			id: 99,
			bitmap: Some(vec![10, 20, 30, 40]),
			width: 64,
			height: 128,
			left: -5,
			top: 10,
			advance: 70,
		};

		let encoded = original_glyph.encode_to_vec();
		let decoded_glyph = Glyph::decode(&encoded[..]).unwrap();

		assert_eq!(
			format!("{decoded_glyph:?}"),
			"Glyph { id: 99, bitmap: Some([10, 20, 30, 40]), width: 64, height: 128, left: -5, top: 10, advance: 70 }"
		);
	}

	#[test]
	fn test_serialization_no_bitmap() {
		let original_glyph = Glyph {
			id: 1,
			bitmap: None,
			width: 12,
			height: 24,
			left: 1,
			top: 2,
			advance: 10,
		};

		let encoded = original_glyph.encode_to_vec();
		let decoded_glyph = Glyph::decode(&encoded[..]).unwrap();

		assert_eq!(
			format!("{decoded_glyph:?}"),
			"Glyph { id: 1, bitmap: None, width: 12, height: 24, left: 1, top: 2, advance: 10 }"
		);
	}
}
