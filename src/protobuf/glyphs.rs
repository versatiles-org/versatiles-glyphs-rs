use super::{fontstack::Fontstack, PbfGlyph};
use anyhow::Result;
use prost::{alloc, Message};

/// A collection of one or more [`Fontstack`] instances,
/// each containing glyph data for a particular font or range.
///
/// This top-level message is often used to represent an entire
/// set of glyphs for multiple fonts in a single protobuf structure.
#[derive(Clone, PartialEq, Message)]
pub struct PbfGlyphs {
	/// A list of [`Fontstack`] objects, where each entry
	/// corresponds to a unique font name or ID range.
	#[prost(message, repeated, tag = "1")]
	stacks: alloc::vec::Vec<Fontstack>,
}

impl PbfGlyphs {
	/// Creates a new [`PbfGlyphs`] for the specified font name and glyph ID `range`.
	///
	/// # Examples
	///
	/// ```
	/// use versatiles_glyphs::protobuf::PbfGlyphs;
	///
	/// let pbf = PbfGlyphs::new("MyFont".to_string(), "0-255".to_string());
	/// ```
	pub fn new(name: String, range: String) -> Self {
		Self {
			stacks: vec![Fontstack::new(name, range)],
		}
	}

	/// Adds a single [`Glyph`] to the wrapped `Fontstack`.
	///
	/// # Examples
	///
	/// ```
	/// use versatiles_glyphs::protobuf::{PbfGlyph, PbfGlyphs};
	///
	/// let mut pbf = PbfGlyphs::new("MyFont".to_string(), "0-255".to_string());
	/// pbf.push(PbfGlyph::empty(42, 12));
	/// ```
	pub fn push(&mut self, glyph: PbfGlyph) {
		self.stacks[0].glyphs.push(glyph);
	}

	/// Consumes this instance, returning a protobuf-encoded representation
	/// of the underlying data in a `Vec<u8>`.
	///
	/// # Errors
	///
	/// Returns an [`anyhow::Error`] if encoding fails.
	///
	/// # Examples
	///
	/// ```
	/// use versatiles_glyphs::protobuf::{PbfGlyph, PbfGlyphs};
	///
	/// let mut pbf = PbfGlyphs::new("MyFont".to_string(), "0-255".to_string());
	/// pbf.push(PbfGlyph::empty(42, 12));
	///
	/// let bytes = pbf.into_vec().unwrap();
	/// assert!(!bytes.is_empty());
	/// ```
	pub fn into_vec(self) -> Result<Vec<u8>> {
		let mut out_buf = Vec::new();
		self.encode(&mut out_buf)?;
		Ok(out_buf)
	}

	/// Consumes this instance, returning a vector of all [`PbfGlyph`] instances
	/// contained within the wrapped `Fontstack`.
	pub fn into_glyphs(self) -> Vec<PbfGlyph> {
		self.stacks.into_iter().flat_map(|fs| fs.glyphs).collect()
	}
}

impl From<Fontstack> for PbfGlyphs {
	fn from(stack: Fontstack) -> Self {
		PbfGlyphs {
			stacks: vec![stack],
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_glyphs_new() {
		let glyphs = PbfGlyphs::from(Fontstack::new("TestFont".to_string(), "0-255".to_string()));

		let encoded_data = glyphs.encode_to_vec();
		let decoded_glyphs = PbfGlyphs::decode(&encoded_data[..]).unwrap();

		assert_eq!(
			format!("{decoded_glyphs:?}"),
			"PbfGlyphs { stacks: [Fontstack { name: \"TestFont\", range: \"0-255\", glyphs: [] }] }"
		);
	}

	#[test]
	fn test_pbf_glyphs_multiple_glyphs() {
		let mut pbf = PbfGlyphs::new("MultiStack".to_string(), "100-200".to_string());

		let glyph_a = PbfGlyph {
			id: 100,
			bitmap: Some(vec![10, 20]),
			width: 15,
			height: 20,
			left: -2,
			top: 5,
			advance: 16,
		};
		let glyph_b = PbfGlyph {
			id: 101,
			bitmap: None,
			width: 9,
			height: 10,
			left: 0,
			top: 2,
			advance: 11,
		};

		pbf.push(glyph_a.clone());
		pbf.push(glyph_b.clone());

		// Serialize
		let data = pbf.into_vec().expect("Failed to encode multiple glyphs");
		// Decode
		let decoded = PbfGlyphs::decode(&data[..]).expect("Failed to decode multiple glyphs");

		let fs = &decoded.stacks[0];
		assert_eq!(fs.glyphs[0], glyph_a);
		assert_eq!(fs.glyphs[1], glyph_b);
		assert_eq!(format!("{decoded:?}"), "PbfGlyphs { stacks: [Fontstack { name: \"MultiStack\", range: \"100-200\", glyphs: [PbfGlyph { id: 100, bitmap: Some([10, 20]), width: 15, height: 20, left: -2, top: 5, advance: 16 }, PbfGlyph { id: 101, bitmap: None, width: 9, height: 10, left: 0, top: 2, advance: 11 }] }] }");
	}
}
