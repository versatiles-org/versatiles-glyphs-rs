use super::{fontstack::Fontstack, glyph::Glyph, glyphs::Glyphs};
use anyhow::Result;
use prost::Message;

/// A wrapper around a single `Fontstack` that provides easy methods
/// to add glyphs and convert them into a serialized protobuf buffer.
pub struct PbfGlyphs {
	fontstack: Fontstack,
}

impl PbfGlyphs {
	/// Creates a new [`PbfGlyphs`] for the specified font name and glyph ID `range`.
	///
	/// # Examples
	///
	/// ```
	/// use versatiles_glyphs::protobuf::pbf_glyphs::PbfGlyphs;
	///
	/// let pbf = PbfGlyphs::new("MyFont".to_string(), "0-255".to_string());
	/// ```
	pub fn new(name: String, range: String) -> Self {
		Self {
			fontstack: Fontstack::new(name, range),
		}
	}

	/// Adds a single [`Glyph`] to the wrapped `Fontstack`.
	///
	/// # Examples
	///
	/// ```
	/// use versatiles_glyphs::protobuf::{glyph::Glyph, pbf_glyphs::PbfGlyphs};
	///
	/// let mut pbf = PbfGlyphs::new("MyFont".to_string(), "0-255".to_string());
	/// pbf.push(Glyph::empty(42, 12));
	/// ```
	pub fn push(&mut self, glyph: Glyph) {
		self.fontstack.glyphs.push(glyph);
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
	/// use versatiles_glyphs::protobuf::{glyph::Glyph, pbf_glyphs::PbfGlyphs};
	///
	/// let mut pbf = PbfGlyphs::new("MyFont".to_string(), "0-255".to_string());
	/// pbf.push(Glyph::empty(42, 12));
	///
	/// let bytes = pbf.into_vec().unwrap();
	/// assert!(!bytes.is_empty());
	/// ```
	pub fn into_vec(self) -> Result<Vec<u8>> {
		let glyphs = Glyphs {
			stacks: vec![self.fontstack],
		};
		let mut out_buf = Vec::new();
		glyphs.encode(&mut out_buf)?;
		Ok(out_buf)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_pbf_glyphs_multiple_glyphs() {
		let mut pbf = PbfGlyphs::new("MultiStack".to_string(), "100-200".to_string());

		let glyph_a = Glyph {
			id: 100,
			bitmap: Some(vec![10, 20]),
			width: 15,
			height: 20,
			left: -2,
			top: 5,
			advance: 16,
		};
		let glyph_b = Glyph {
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
		let decoded: Glyphs = Glyphs::decode(&data[..]).expect("Failed to decode multiple glyphs");

		let fs = &decoded.stacks[0];
		assert_eq!(fs.glyphs[0], glyph_a);
		assert_eq!(fs.glyphs[1], glyph_b);
		assert_eq!(format!("{decoded:?}"), "Glyphs { stacks: [Fontstack { name: \"MultiStack\", range: \"100-200\", glyphs: [Glyph { id: 100, bitmap: Some([10, 20]), width: 15, height: 20, left: -2, top: 5, advance: 16 }, Glyph { id: 101, bitmap: None, width: 9, height: 10, left: 0, top: 2, advance: 11 }] }] }");
	}
}
