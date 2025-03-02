use super::font_metadata::FontMetadata;
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
