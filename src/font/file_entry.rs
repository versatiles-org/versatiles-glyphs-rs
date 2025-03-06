use super::metadata::FontMetadata;
use anyhow::{Context, Result};
use std::{marker::PhantomPinned, pin::Pin, slice};
use ttf_parser::Face;

#[derive(Debug)]
/// A font file entry contains the raw bytes of a font file, its parsed face and the metadata of the font.
/// It is used as a wrapper to handle multiple references to the same font file in memory.
pub struct FontFileEntry<'a> {
	#[allow(dead_code)]
	data: Pin<Vec<u8>>,
	pub face: Face<'a>,
	pub metadata: FontMetadata,
	_pin: PhantomPinned,
}

impl<'a> FontFileEntry<'a> {
	pub fn new(data: Vec<u8>) -> Result<Self> {
		unsafe {
			let data = Pin::new(data);
			let slice: &'a [u8] = slice::from_raw_parts(data.as_ptr(), data.len());
			let face = Face::parse(slice, 0).context("Could not parse font data")?;
			let metadata = FontMetadata::try_from(&face)?;
			Ok(FontFileEntry {
				data,
				face,
				metadata,
				_pin: PhantomPinned,
			})
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	const FIRA: &[u8] = include_bytes!("../../testdata/Fira Sans - Regular.ttf");

	#[test]
	fn test_font_file_entry_new_with_valid_font() {
		let data = FIRA.to_vec();
		let entry = FontFileEntry::new(data).unwrap();
		assert_eq!(entry.face.number_of_glyphs(), 2677);
		assert_eq!(entry.metadata.generate_name(), "Fira Sans Regular");
	}

	#[test]
	fn test_font_file_entry_new_with_invalid_font() {
		let invalid_data = vec![0x00, 0x01, 0x02];
		let result = FontFileEntry::new(invalid_data);
		assert_eq!(result.unwrap_err().to_string(), "Could not parse font data");
	}
}
