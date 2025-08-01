use super::{wrapper::FontWrapper, FontMetadata};
use anyhow::Result;
use std::collections::{HashMap, HashSet};

/// Data structure representing a single font face within a family.
/// This includes the unique `id`, as well as styling attributes like
/// `style`, `weight`, and `width`.
#[derive(serde::Serialize)]
struct FontFace {
	id: String,
	style: String,
	weight: u16,
	width: String,
	codeblocks: String,
}

/// Data structure representing a font family, which can contain
/// one or more [`FontFace`] entries.
#[derive(serde::Serialize)]
struct FontFamily {
	/// Name of the font family, e.g., "Noto Sans".
	name: String,
	/// Collection of faces that belong to this family.
	faces: Vec<FontFace>,
}

impl FontFamily {
	/// Creates a new font family with the given name.
	fn new(name: String) -> Self {
		FontFamily {
			name,
			faces: Vec::new(),
		}
	}

	/// Adds a new [`FontFace`] to this family.
	fn add_font(&mut self, id: String, meta: &FontMetadata) {
		self.faces.push(FontFace {
			id,
			style: meta.style.clone(),
			weight: meta.weight,
			width: meta.width.clone(),
			codeblocks: encode_codeblocks(&meta.codepoints),
		});
	}
}

/// Builds a compact, comma-separated string of all the 16-codepoint blocks
/// spanned by the provided Unicode codepoints. Each 16-codepoint “block” is
/// identified by `(codepoint >> 4)`—i.e., the upper bits beyond the last 4 bits.
///
/// See also: https://www.unicode.org/Public/UCD/latest/ucd/Blocks.txt
///
/// # How It Works
/// 1. Every codepoint `cp` is mapped to `cp >> 4`, grouping sets of 16 consecutive codepoints (e.g., `0x0000..=0x000F` = block `0x0`).
/// 2. Consecutive block indices are merged into ranges. For instance, blocks `0,1,2,3` become `"0-3"`.
/// 3. Formatting:
///    - Each block index is written in uppercase hex (`{:X}`).
///    - Single blocks appear as e.g. `"A2"` rather than `"A2-A2"`.
///    - Multiple blocks or ranges are joined with commas (e.g. `"0-3,5,A-C"`).
///
/// # Return Value
/// A string of comma-separated hexadecimal block ranges, or an empty string if
/// no codepoints are provided.
fn encode_codeblocks(codepoints: &[u32]) -> String {
	let blocks = HashSet::<u32>::from_iter(codepoints.iter().map(|&cp| cp >> 4));
	let mut blocks: Vec<u32> = blocks.into_iter().collect();
	blocks.sort_unstable();

	if blocks.is_empty() {
		return String::new();
	}

	// Group consecutive block indices into ranges.
	let mut ranges = Vec::new();
	let (mut start, mut prev) = (blocks[0], blocks[0]);
	for &block in &blocks[1..] {
		if block != prev + 1 {
			ranges.push((start, prev));
			start = block;
		}
		prev = block;
	}
	// Push the final range
	ranges.push((start, prev));

	// Format each range as uppercase hex.
	ranges
		.iter()
		.map(|&(s, e)| {
			if s == e {
				format!("{s:X}")
			} else {
				format!("{s:X}-{e:X}")
			}
		})
		.collect::<Vec<_>>()
		.join(",")
}

/// Builds an index (list) of all font IDs, returning JSON-encoded bytes.
///
/// The iterator should yield `(id, FontWrapper)` pairs. The resulting JSON
/// is an array of sorted string IDs.
///
/// # Errors
///
/// Returns an error if the encoding process fails.
pub fn build_index_json<'a>(iter: impl Iterator<Item = &'a String>) -> Result<Vec<u8>> {
	let mut list = iter.collect::<Vec<_>>();
	list.sort();
	Ok(serde_json::to_vec_pretty(&list)?)
}

/// Builds a list of font families, each containing one or more font faces,
/// returning JSON-encoded bytes.
///
/// The iterator should yield `(id, FontWrapper)` pairs. Each font's
/// metadata is examined, and faces with the same family name are grouped together.
/// The JSON contains a sorted array of families, each with an array of faces.
///
/// # Errors
///
/// Returns an error if the encoding process fails.
pub fn build_font_families_json<'a>(
	iter: impl Iterator<Item = (&'a String, &'a FontWrapper<'a>)>,
) -> Result<Vec<u8>> {
	let mut family_map = HashMap::<String, FontFamily>::new();
	for (id, font) in iter {
		let meta = font.get_metadata();
		family_map
			.entry(meta.family.to_string())
			.or_insert_with(|| FontFamily::new(meta.family.to_string()))
			.add_font(id.to_string(), meta);
	}
	let mut families = family_map.into_values().collect::<Vec<_>>();
	families.sort_by_cached_key(|f| f.name.clone());
	Ok(serde_json::to_vec_pretty(&families)?)
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::font::FontManager;
	use std::path::PathBuf;

	#[test]
	fn test_build_index_json() -> Result<()> {
		let mut manager = FontManager::new(false);
		manager.add_paths(&[
			PathBuf::from("./testdata/Fira Sans - Regular.ttf"),
			PathBuf::from("./testdata/Noto Sans/Noto Sans - Regular.ttf"),
		])?;

		let json_bytes = build_index_json(manager.fonts.keys())?;
		assert_eq!(
			String::from_utf8(json_bytes)?
				.split('\n')
				.collect::<Vec<_>>(),
			[
				"[",
				"  \"fira_sans_regular\",",
				"  \"noto_sans_regular\"",
				"]"
			]
		);
		Ok(())
	}

	#[test]
	fn test_build_font_families_json() -> Result<()> {
		let mut manager = FontManager::new(false);
		manager.add_paths(&[
			PathBuf::from("./testdata/Fira Sans - Regular.ttf"),
			PathBuf::from("./testdata/Noto Sans/Noto Sans - Regular.ttf"),
		])?;

		let json_bytes = build_font_families_json(manager.fonts.iter())?;
		assert_eq!(
			String::from_utf8(json_bytes)?
				.split('\n')
				.collect::<Vec<_>>(),
			[
				"[", 
				"  {", 
				"    \"name\": \"Fira Sans\",", 
				"    \"faces\": [", 
				"      {", 
				"        \"id\": \"fira_sans_regular\",", 
				"        \"style\": \"normal\",", 
				"        \"weight\": 400,", 
				"        \"width\": \"normal\",", 
				"        \"codeblocks\": \"0,2-7,A-2E,30-52,E3,1D4,1D6-1D7,1D9,1DB-1DC,1E0-204,207-208,20A-20B,210-212,215,219,21E,220-222,224,226,22C,232,23C,25A,25C,2C6-2C7,A78,A7A-A7B,AB5,FB0,FEF\"", 
				"      }", 
				"    ]", 
				"  },", 
				"  {", 
				"    \"name\": \"Noto Sans\",", 
				"    \"faces\": [", 
				"      {", 
				"        \"id\": \"noto_sans_regular\",", 
				"        \"style\": \"normal\",", 
				"        \"weight\": 400,", 
				"        \"width\": \"normal\",", 
				"        \"codeblocks\": \"0,2-7,A-52,90-97,10F,1AB-1AC,1C8,1D0-20C,20F-215,218,221,25C,2C6-2C7,2DE-2E5,A64-A69,A70-A7D,A7F,A8F,A92,AB3-AB6,FB0,FE0,FE2,FEF,FFF,1078-107B,1DF0-1DF1\"", 
				"      }", 
				"    ]", 
				"  }", 
				"]"
			]
		);
		Ok(())
	}

	#[test]
	fn empty_input_returns_empty_string() {
		assert_eq!(encode_codeblocks(&[]), "");
	}

	#[test]
	fn single_codepoint_yields_one_block() {
		assert_eq!(encode_codeblocks(&[0xA3]), "A");
	}

	#[test]
	fn consecutive_codepoints_merge_blocks() {
		assert_eq!(encode_codeblocks(&[0x0, 0x1, 0x2, 0xF, 0x10]), "0-1");
	}

	#[test]
	fn disjoint_blocks_produce_multiple_ranges() {
		assert_eq!(encode_codeblocks(&[0x0, 0x2, 0x1F, 0x40, 0xA0]), "0-1,4,A");
	}
}
