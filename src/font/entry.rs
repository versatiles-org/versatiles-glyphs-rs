use super::{load_font_metadata, metadata::FaceMetadata};
use anyhow::{Context, Result};
use std::{marker::PhantomPinned, pin::Pin, slice};
use ttf_parser::Face;

pub struct FontEntry<'a> {
	#[allow(dead_code)]
	data: Pin<Vec<u8>>,
	pub face: Face<'a>,
	pub metadata: FaceMetadata,
	_pin: PhantomPinned,
}

impl<'a> FontEntry<'a> {
	pub fn new(data: Vec<u8>) -> Result<Self> {
		unsafe {
			let data = Pin::new(data);
			let slice: &'a [u8] = slice::from_raw_parts(data.as_ptr(), data.len());
			let face = Face::parse(slice, 0).context("Could not parse font data")?;
			let metadata = load_font_metadata(&face)?;
			Ok(FontEntry {
				data,
				face,
				metadata,
				_pin: PhantomPinned,
			})
		}
	}
}
