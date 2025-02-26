use super::{PbfFontstack, PbfGlyph};
use anyhow::{anyhow, Result};
use prost::Message;
use std::collections::HashMap;

/// Merge multiple PBF buffers with glyph data into one combined PBF.
pub fn composite(glyph_buffers: &[Vec<u8>]) -> Result<Vec<u8>> {
	let mut merged_fontstack = PbfFontstack::default();
	let mut seen_glyphs = HashMap::<u32, PbfGlyph>::new();
	let mut is_first = true;

	for buf in glyph_buffers {
		// Decode a FontStack
		let fontstack = PbfFontstack::decode(buf.as_slice())
			.map_err(|_| anyhow!("Failed to parse FontStack from buffer"))?;

		// Merge the name & range if it’s the first buffer, or append names
		if is_first {
			merged_fontstack.name = fontstack.name.clone();
			merged_fontstack.range = fontstack.range.clone();
			is_first = false;
		} else {
			// If you want to replicate the “fontstack_name = fontstack_name + ", " + next_name” logic:
			merged_fontstack.name = format!("{}, {}", merged_fontstack.name, fontstack.name);
			// Typically, we keep the same range from the first buffer or build a union, etc.
		}

		// Merge glyphs: if we have not seen a glyph_id, add it
		for g in fontstack.glyphs {
			seen_glyphs.entry(g.id).or_insert(g);
		}
	}

	// Convert map -> vector
	for (_cp, glyph) in seen_glyphs {
		merged_fontstack.glyphs.push(glyph);
	}

	// Encode final merged PBF
	let mut out_buf = Vec::new();
	merged_fontstack.encode(&mut out_buf)?;
	Ok(out_buf)
}
