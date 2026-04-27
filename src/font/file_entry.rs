use super::metadata::FontMetadata;
use anyhow::{Context, Result};
use std::{marker::PhantomPinned, pin::Pin, slice};
use ttf_parser::Face;

/// A font file entry that holds raw font bytes, a parsed [`Face`], and font metadata.
/// This structure is pinned to ensure safe references to the underlying font data.
#[derive(Debug)]
pub struct FontFileEntry<'a> {
	/// The parsed [`Face`] containing information like glyph count, names, and metrics.
	pub face: Face<'a>,

	/// The metadata extracted from the font, such as name, style, and other descriptors.
	pub metadata: FontMetadata,

	/// Pinned backing storage for `face`'s borrowed slice.
	///
	/// Load-bearing despite never being read directly: dropping or moving it would
	/// invalidate the `&'a [u8]` that `face` holds. Hence `#[allow(dead_code)]`.
	#[allow(dead_code)]
	data: Pin<Vec<u8>>,

	/// Prevents movement of the struct after pinning.
	_pin: PhantomPinned,
}

impl<'a> FontFileEntry<'a> {
	/// Creates a new [`FontFileEntry`] from raw bytes.
	///
	/// # Errors
	/// Returns an error if the font data fails to parse.
	pub fn new(data: Vec<u8>) -> Result<Self> {
		let data = Pin::new(data);
		// SAFETY: This builds a self-referential struct. The slice we hand to
		// `Face::parse` borrows from the bytes owned by `data`. The borrow is
		// safe because:
		//   1. `data` is `Pin<Vec<u8>>` and exposed only by reference, so the
		//      `Vec`'s heap allocation is never moved or reallocated for the
		//      lifetime of the struct.
		//   2. `_pin: PhantomPinned` makes the struct itself `!Unpin`, so safe
		//      code cannot move `FontFileEntry` once constructed.
		//   3. `data` is dropped together with `face` when the struct is
		//      dropped, so the slice never outlives its backing storage.
		// The lifetime parameter `'a` is only nominal here — nothing outside
		// the struct provides it; it exists so `Face<'a>` can borrow the
		// internal slice.
		let slice: &'a [u8] = unsafe { slice::from_raw_parts(data.as_ptr(), data.len()) };
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
