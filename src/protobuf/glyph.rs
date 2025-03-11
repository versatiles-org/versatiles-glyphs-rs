use prost::{alloc, Message};

/// A representation of an individual glyph, complete with bitmap data and metrics.
///
/// Each glyph is identified by an `id` and may have an optional `bitmap` buffer
/// that holds its Signed Distance Field data. The dimensions and metrics (width,
/// height, left, top, and advance) describe how this glyph is displayed and placed
/// in a layout.
#[derive(Clone, PartialEq, Message)]
pub struct PbfGlyph {
	/// The numeric identifier corresponding to this glyph.
	#[prost(uint32, required, tag = "1")]
	pub id: u32,

	/// Optional Signed Distance Field for the glyph, potentially including a
	/// 3-pixel border to capture smoothing data.
	#[prost(bytes = "vec", optional, tag = "2")]
	pub bitmap: Option<alloc::vec::Vec<u8>>,

	/// The width of the glyph bitmap, in pixels.
	#[prost(uint32, required, tag = "3")]
	pub width: u32,

	/// The height of the glyph bitmap, in pixels.
	#[prost(uint32, required, tag = "4")]
	pub height: u32,

	/// The number of pixels to move to the right before drawing the bitmap,
	/// relative to the drawing cursor.
	#[prost(sint32, required, tag = "5")]
	pub left: i32,

	/// The number of pixels to move down before drawing the bitmap,
	/// relative to the baseline.
	#[prost(sint32, required, tag = "6")]
	pub top: i32,

	/// The horizontal distance to advance the cursor after drawing this glyph.
	#[prost(uint32, required, tag = "7")]
	pub advance: u32,
}

impl PbfGlyph {
	/// Creates a new [`Glyph`] with the specified `id` and `advance`,
	/// while leaving all other fields unset or zero.
	///
	/// This is useful for glyphs that only need advance/position metrics
	/// without any bitmap data.
	///
	/// # Examples
	///
	/// ```
	/// use versatiles_glyphs::protobuf::Glyph;
	///
	/// let glyph = Glyph::empty(42, 100);
	/// assert_eq!(glyph.id, 42);
	/// assert_eq!(glyph.advance, 100);
	/// assert!(glyph.bitmap.is_none());
	/// ```
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_empty_glyph() {
		let glyph_id = 42;
		let advance = 100;
		let original_glyph = PbfGlyph::empty(glyph_id, advance);

		let encoded = original_glyph.encode_to_vec();
		let decoded_glyph = PbfGlyph::decode(&encoded[..]).unwrap();

		assert_eq!(
			format!("{decoded_glyph:?}"),
			"PbfGlyph { id: 42, bitmap: None, width: 0, height: 0, left: 0, top: 0, advance: 100 }"
		);
	}

	#[test]
	fn test_serialization_round_trip() {
		let original_glyph = PbfGlyph {
			id: 99,
			bitmap: Some(vec![10, 20, 30, 40]),
			width: 64,
			height: 128,
			left: -5,
			top: 10,
			advance: 70,
		};

		let encoded = original_glyph.encode_to_vec();
		let decoded_glyph = PbfGlyph::decode(&encoded[..]).unwrap();

		assert_eq!(
            format!("{decoded_glyph:?}"),
            "PbfGlyph { id: 99, bitmap: Some([10, 20, 30, 40]), width: 64, height: 128, left: -5, top: 10, advance: 70 }"
        );
	}

	#[test]
	fn test_serialization_no_bitmap() {
		let original_glyph = PbfGlyph {
			id: 1,
			bitmap: None,
			width: 12,
			height: 24,
			left: 1,
			top: 2,
			advance: 10,
		};

		let encoded = original_glyph.encode_to_vec();
		let decoded_glyph = PbfGlyph::decode(&encoded[..]).unwrap();

		assert_eq!(
			format!("{decoded_glyph:?}"),
			"PbfGlyph { id: 1, bitmap: None, width: 12, height: 24, left: 1, top: 2, advance: 10 }"
		);
	}
}
