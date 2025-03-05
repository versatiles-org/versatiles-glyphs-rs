use std::collections::HashMap;

use anyhow::Result;

use super::renderer::FontRenderer;

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
	iter:  impl Iterator<Item = (&'a String, &'a FontRenderer<'a>)>,
) -> Result<Vec<u8>> {
	let mut list = iter.map(|f| f.0.clone()).collect::<Vec<_>>();
	list.sort();
	Ok(serde_json::to_vec_pretty(&list)?)
}

pub fn build_font_families_json<'a>(
	iter:  impl Iterator<Item = (&'a String, &'a FontRenderer<'a>)>,
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
