use anyhow::{anyhow, Context, Result};
use ttf_parser::Face;

#[derive(Debug)]
pub struct FaceMetadata {
	pub family_name: String,
	pub style_name: Option<String>,
	pub points: Vec<u32>,
}

impl FaceMetadata {
	pub fn new(family: String, style: Option<String>, points: Vec<u32>) -> Self {
		Self {
			family_name: family,
			style_name: style,
			points,
		}
	}
}

// ---------------------------------------------------------
// 1) LOAD: analogous to AsyncLoad / Load(...)
//    Pure Rust version using ttf-parser
// ---------------------------------------------------------

pub fn load_font(data: &[u8]) -> Result<Vec<FaceMetadata>> {
	// ttf-parser doesn’t automatically parse multi-face fonts
	// (like a TrueType Collection).
	// For single-face fonts, do:
	let face = Face::parse(data, 0).context("Could not parse font data")?;

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
	let style_name = face.names().into_iter().find_map(|n| {
		if n.name_id == ttf_parser::name_id::SUBFAMILY {
			n.to_string()
		} else {
			None
		}
	});

	// Enumerate all codepoints in the font’s cmap:
	let mut codepoints = Vec::new();
	let table = face
		.tables()
		.cmap
		.ok_or(anyhow!("Font has no cmap table"))?;
	for subtable in table.subtables.into_iter() {
		subtable.codepoints(|cp| codepoints.push(cp));
	}

	// Build FaceMetadata
	let meta = FaceMetadata::new(family_name, style_name, codepoints);
	Ok(vec![meta])
}
