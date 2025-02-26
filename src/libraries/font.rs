use anyhow::{anyhow, Result};
use ttf_parser::{Face, PlatformId};

#[derive(Debug)]
pub struct FaceMetadata {
	pub family_name: String,
	pub style_name: String,
	pub codepoints: Vec<u32>,
}

// ---------------------------------------------------------
// 1) LOAD: analogous to AsyncLoad / Load(...)
//    Pure Rust version using ttf-parser
// ---------------------------------------------------------

pub fn load_font_metadata(face: &Face) -> Result<FaceMetadata> {
	// Try to gather a "family_name" from the name table
	let family_name = face
		.names()
		.into_iter()
		.find_map(|name| {
			// This tries to find the "Full Name" or "Family Name".
			// Adjust name_id as needed
			if name.name_id == ttf_parser::name_id::FULL_NAME
				|| name.name_id == ttf_parser::name_id::FAMILY
			{
				name.to_string()
			} else {
				None
			}
		})
		.unwrap_or_else(|| "UnknownFamily".to_string());

	// Style name, e.g. Subfamily
	let style_name = face
		.names()
		.into_iter()
		.find_map(|n| {
			if n.name_id == ttf_parser::name_id::SUBFAMILY {
				n.to_string()
			} else {
				None
			}
		})
		.unwrap_or(String::from("unknown"));

	// Enumerate all codepoints in the font’s cmap:
	let mut codepoints = Vec::new();
	let table = face
		.tables()
		.cmap
		.ok_or(anyhow!("Font has no cmap table"))?;

	for subtable in table.subtables.into_iter() {
		if subtable.platform_id != PlatformId::Unicode {
			continue;
		}
		subtable.codepoints(|cp| codepoints.push(cp));
	}

	// Build FaceMetadata
	let meta = FaceMetadata {
		family_name,
		style_name,
		codepoints,
	};
	Ok(meta)
}
