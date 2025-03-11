//! Metadata extraction and analysis for font files.
//!
//! This module defines [`FontMetadata`] and provides functionality for
//! gathering name, family, style, weight, width, and codepoint coverage
//! information from a [`ttf_parser::Face`].

use anyhow::Result;
use std::{
	collections::{HashMap, HashSet},
	fmt::Debug,
};
use ttf_parser::{name_id, Face, PlatformId};

use super::parse_font_name;

#[allow(dead_code)]
/// Stores extracted font properties such as `family`, `style`, and `weight`,
/// along with a set of all supported codepoints.
pub struct FontMetadata {
	/// The raw font name (may include style and other descriptors).
	pub name: String,
	/// The family portion of the font name (e.g. "Noto Sans").
	pub family: String,
	/// All codepoints supported by this font.
	pub codepoints: Vec<u32>,
	/// Font style, typically "normal" or "italic".
	pub style: String,
	/// Numerical weight (e.g. 400 for Regular, 700 for Bold, etc.).
	pub weight: u16,
	/// Width descriptor, often "normal", "condensed", or "expanded".
	pub width: String,
}

impl FontMetadata {
	/// Generates a human-readable name, including family, width, weight, and style.
	///
	/// For example, a font with `family = "Noto Sans"`, `weight = 400`,
	/// `width = "normal"`, and `style = "normal"` produces `"Noto Sans Regular"`.
	pub fn generate_name(&self) -> String {
		let mut name = self.family.clone();
		if self.width != "normal" {
			name = format!("{name} {}", self.width);
		}

		let weight = match self.weight {
			100 => "Thin",
			200 => "ExtraLight",
			300 => "Light",
			400 => "Regular",
			500 => "Medium",
			600 => "SemiBold",
			700 => "Bold",
			800 => "ExtraBold",
			900 => "Black",
			_ => "Unknown",
		};
		name = format!("{name} {weight}");

		if self.style != "normal" {
			name = format!("{name} {}", self.style);
		}

		name
	}
}

impl Debug for FontMetadata {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"FontMetadata {{ family: {}, style: {}, weight: {}, width: {}, codepoints: {} }}",
			self.family,
			self.style,
			self.weight,
			self.width,
			self.codepoints.len()
		)
	}
}

impl TryFrom<&Face<'_>> for FontMetadata {
	type Error = anyhow::Error;

	/// Attempts to build a [`FontMetadata`] from a [`ttf_parser::Face`],
	/// extracting properties like family name, style, weight, width, and codepoints.
	fn try_from(face: &Face) -> Result<Self> {
		let map = HashMap::<u16, String>::from_iter(
			face
				.names()
				.into_iter()
				.map(|n| (n.name_id, n.to_string().unwrap_or_default())),
		);

		let get = |id: u16| map.get(&id).unwrap_or(&String::from("")).to_owned();

		let name = get(name_id::FAMILY);
		let (family, style, weight, width) =
			parse_font_name(name.clone(), get(name_id::POST_SCRIPT_NAME));

		let mut codepoints = HashSet::<u32>::new();
		let table = face.tables().cmap.expect("Font has no cmap table");
		for subtable in table.subtables.into_iter() {
			if subtable.is_unicode() {
				subtable.codepoints(|cp| {
					if subtable.glyph_index(cp).is_some() {
						codepoints.insert(cp);
					}
				});
			}
		}
		let mut codepoints = codepoints.into_iter().collect::<Vec<u32>>();
		codepoints.sort_unstable();

		let metadata = FontMetadata {
			name,
			family,
			codepoints,
			style,
			weight,
			width,
		};

		Ok(metadata)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_load_fira() {
		const FIRA: &[u8] = include_bytes!("../../testdata/Fira Sans - Regular.ttf");
		let face = Face::parse(FIRA, 0).unwrap();
		let metadata = FontMetadata::try_from(&face).unwrap();
		assert_eq!(metadata.family, "Fira Sans");
		assert_eq!(metadata.generate_name(), "Fira Sans Regular");
		assert_eq!(metadata.codepoints.len(), 1686);
	}

	#[test]
	fn test_load_noto() {
		const NOTO: &[u8] = include_bytes!("../../testdata/Noto Sans/Noto Sans - Regular.ttf");
		let face = Face::parse(NOTO, 0).unwrap();
		let metadata = FontMetadata::try_from(&face).unwrap();
		assert_eq!(metadata.family, "Noto Sans");
		assert_eq!(metadata.generate_name(), "Noto Sans Regular");
		assert_eq!(metadata.codepoints.len(), 3094);
	}
}
