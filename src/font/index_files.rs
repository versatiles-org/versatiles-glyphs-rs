use super::wrapper::FontWrapper;
use anyhow::Result;
use std::collections::HashMap;

/// Data structure representing a single font face within a family.
/// This includes the unique `id`, as well as styling attributes like
/// `style`, `weight`, and `width`.
#[derive(serde::Serialize)]
struct FontFace {
	id: String,
	style: String,
	weight: u16,
	width: String,
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
	fn add_font(&mut self, id: String, style: String, weight: u16, width: String) {
		self.faces.push(FontFace {
			id,
			style,
			weight,
			width,
		});
	}
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
			.add_font(
				id.to_string(),
				meta.style.to_string(),
				meta.weight,
				meta.width.to_string(),
			);
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
		let mut manager = FontManager::default();
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
		let mut manager = FontManager::default();
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
				"        \"width\": \"normal\"",
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
				"        \"width\": \"normal\"",
				"      }",
				"    ]",
				"  }",
				"]"
			]
		);
		Ok(())
	}
}
