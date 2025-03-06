use anyhow::Result;
use std::{collections::HashMap, fmt::Debug};
use ttf_parser::{name_id, Face, PlatformId};

use super::parse_font_name;

pub struct FontMetadata {
	pub family: String,
	pub codepoints: Vec<u32>,
	pub style: String,
	pub weight: u16,
	pub width: String,
}

impl FontMetadata {
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
		name = format!("{name} {}", weight);

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
	fn try_from(face: &Face) -> Result<Self> {
		let map = HashMap::<u16, String>::from_iter(
			face
				.names()
				.into_iter()
				.map(|n| (n.name_id, n.to_string().unwrap_or_default())),
		);

		let get = |id: u16| map.get(&id).unwrap_or(&String::from("")).to_owned();

		let (family, style, weight, width) =
			parse_font_name(get(name_id::FAMILY), get(name_id::POST_SCRIPT_NAME));

		let mut codepoints = Vec::new();
		let table = face.tables().cmap.expect("Font has no cmap table");

		for subtable in table.subtables.into_iter() {
			if subtable.platform_id != PlatformId::Unicode {
				continue;
			}
			subtable.codepoints(|cp| codepoints.push(cp));
		}

		let metadata = FontMetadata {
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
		assert_eq!(metadata.codepoints.len(), 6100);
	}
}
