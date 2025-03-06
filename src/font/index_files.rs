use super::wrapper::FontWrapper;
use anyhow::Result;
use std::collections::HashMap;

#[derive(serde::Serialize)]
struct FontFace {
	id: String,
	style: String,
	weight: u16,
	width: String,
}

#[derive(serde::Serialize)]
struct FontFamily {
	name: String,
	faces: Vec<FontFace>,
}

impl FontFamily {
	fn new(name: String) -> FontFamily {
		FontFamily {
			name,
			faces: Vec::new(),
		}
	}

	fn add_font(&mut self, id: String, style: String, weight: u16, width: String) {
		self.faces.push(FontFace {
			id,
			style,
			weight,
			width,
		});
	}
}

pub fn build_index_json<'a>(
	iter: impl Iterator<Item = (&'a String, &'a FontWrapper<'a>)>,
) -> Result<Vec<u8>> {
	let mut list = iter.map(|f| f.0.clone()).collect::<Vec<_>>();
	list.sort();
	Ok(serde_json::to_vec_pretty(&list)?)
}

pub fn build_font_families_json<'a>(
	iter: impl Iterator<Item = (&'a String, &'a FontWrapper<'a>)>,
) -> Result<Vec<u8>> {
	let mut family_map = HashMap::<String, FontFamily>::new();
	for (id, renderer) in iter {
		let meta = renderer.get_metadata();
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

		let json_bytes = build_index_json(manager.fonts.iter())?;
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
