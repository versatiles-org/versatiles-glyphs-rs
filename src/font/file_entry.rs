use super::{
	arabic_forms::{synthesize_arabic_forms, SynthesizedGlyph},
	metadata::FontMetadata,
};
use crate::{protobuf::PbfGlyph, render::Renderer};
use anyhow::{Context, Result};
use std::{
	collections::BTreeMap,
	hash::{DefaultHasher, Hash, Hasher},
	marker::PhantomPinned,
	pin::Pin,
	slice,
};
use ttf_parser::Face;

/// A font file entry that holds raw font bytes, a parsed [`Face`], and font metadata.
/// This structure is pinned to ensure safe references to the underlying font data.
#[derive(Debug)]
pub struct FontFileEntry<'a> {
	/// The parsed [`Face`] containing information like glyph count, names, and metrics.
	pub face: Face<'a>,

	/// The metadata extracted from the font, such as name, style, and other descriptors.
	pub metadata: FontMetadata,

	/// Hash of the raw font bytes, identifying the source file by its content.
	///
	/// Two entries with the same hash contain the same font, so they render identical
	/// glyphs. Used to detect ranges that are identical across fonts.
	pub source_hash: u64,

	/// Arabic presentation forms the font does not map, drawn from its GSUB and GPOS.
	///
	/// Their code points are part of [`FontMetadata::codepoints`]. See
	/// [`synthesize_arabic_forms`].
	pub synthesized: BTreeMap<u32, SynthesizedGlyph>,

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
		let mut hasher = DefaultHasher::new();
		data.hash(&mut hasher);
		let source_hash = hasher.finish();

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
		let mut metadata = FontMetadata::try_from(&face)?;
		let synthesized = synthesize_arabic_forms(&face);
		if !synthesized.is_empty() {
			metadata.codepoints.extend(synthesized.keys());
			metadata.codepoints.sort_unstable();
		}
		Ok(FontFileEntry {
			data,
			face,
			metadata,
			source_hash,
			synthesized,
			_pin: PhantomPinned,
		})
	}

	/// Renders the glyph for `codepoint`, from the font's `cmap` or, for an Arabic
	/// presentation form the font does not map, from its [`Self::synthesized`] glyphs.
	///
	/// Returns [`None`] if the font has no glyph for `codepoint`.
	pub fn render_glyph(&self, renderer: &Renderer, codepoint: u32) -> Option<PbfGlyph> {
		match self.synthesized.get(&codepoint) {
			Some(glyph) => {
				Some(renderer.render_parts(&self.face, codepoint, &glyph.parts, glyph.advance))
			}
			None => renderer.render_glyph(&self.face, codepoint),
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
	fn test_source_hash_identifies_content() {
		let noto = include_bytes!("../../testdata/Noto Sans/Noto Sans - Regular.ttf");
		let a = FontFileEntry::new(FIRA.to_vec()).unwrap();
		let b = FontFileEntry::new(FIRA.to_vec()).unwrap();
		let c = FontFileEntry::new(noto.to_vec()).unwrap();
		assert_eq!(a.source_hash, b.source_hash);
		assert_ne!(a.source_hash, c.source_hash);
	}

	#[test]
	fn test_synthesized_forms_are_rendered() {
		let font = include_bytes!("../../testdata/Arabic Forms Test - Regular.ttf");
		let entry = FontFileEntry::new(font.to_vec()).unwrap();
		assert!(entry.synthesized.contains_key(&0xFE92));
		assert!(entry.metadata.codepoints.contains(&0xFE92));
		assert!(entry.metadata.codepoints.is_sorted());

		let renderer = Renderer::new_precise();
		let medial = entry.render_glyph(&renderer, 0xFE92).unwrap();
		let isolated = entry.render_glyph(&renderer, 0x0628).unwrap();
		assert_eq!(medial.id, 0xFE92);
		assert!(medial.bitmap.is_some());
		assert_ne!(medial.bitmap, isolated.bitmap);
		assert!(FontFileEntry::new(FIRA.to_vec())
			.unwrap()
			.synthesized
			.is_empty());
	}

	#[test]
	fn test_synthesized_forms_match_the_mapped_glyphs() {
		// The same font with and without its presentation forms in `cmap`.
		let gsub_only = include_bytes!("../../testdata/Arabic Forms Test - Regular.ttf");
		let mapped = include_bytes!("../../testdata/Noto Sans/Noto Sans Arabic - Regular.ttf");
		let gsub_only = FontFileEntry::new(gsub_only.to_vec()).unwrap();
		let mapped = FontFileEntry::new(mapped.to_vec()).unwrap();
		let renderer = Renderer::new_precise();

		let mut differ = Vec::new();
		for &codepoint in gsub_only.synthesized.keys() {
			let expected = mapped.face.glyph_index(char::from_u32(codepoint).unwrap());
			assert!(expected.is_some(), "{codepoint:04X}");
			if gsub_only.render_glyph(&renderer, codepoint)
				!= mapped.render_glyph(&renderer, codepoint)
			{
				differ.push(codepoint);
			}
		}
		assert_eq!(gsub_only.synthesized.len(), 717);

		// Only forms that Noto draws with a dedicated ligature glyph differ: mark ligatures
		// with a space (U+FC5E–U+FC63), and a few ligatures of letters and whole words.
		assert_eq!(
			differ,
			[
				0xFC5B, 0xFC5C, 0xFC5E, 0xFC5F, 0xFC60, 0xFC61, 0xFC62, 0xFC63, 0xFCD9, 0xFCF2, 0xFCF3,
				0xFCF4, 0xFD3C, 0xFD3D, 0xFDF0, 0xFDF4, 0xFDF7, 0xFDFA, 0xFDFB, 0xFDFC
			]
		);
	}

	#[test]
	fn test_font_file_entry_new_with_invalid_font() {
		let invalid_data = vec![0x00, 0x01, 0x02];
		let result = FontFileEntry::new(invalid_data);
		assert_eq!(result.unwrap_err().to_string(), "Could not parse font data");
	}
}
