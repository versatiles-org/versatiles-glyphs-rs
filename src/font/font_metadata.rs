use anyhow::Result;
use ttf_parser::{name_id, Face, PlatformId};

#[derive(Debug)]
#[allow(unused)]
pub struct FontMetadata {
	pub family_name: String,
	pub style_name: String,
	pub codepoints: Vec<u32>,
}

impl FontMetadata {}

impl TryFrom<&Face<'_>> for FontMetadata {
	type Error = anyhow::Error;
	fn try_from(face: &Face) -> Result<Self> {
		let family_name = face
			.names()
			.into_iter()
			.find_map(|name| {
				if name.name_id == name_id::FULL_NAME || name.name_id == name_id::FAMILY {
					name.to_string()
				} else {
					None
				}
			})
			.unwrap_or_else(|| "UnknownFamily".to_string());

		let style_name = face
			.names()
			.into_iter()
			.find_map(|n| {
				if n.name_id == name_id::SUBFAMILY {
					n.to_string()
				} else {
					None
				}
			})
			.unwrap_or(String::from("unknown"));

		let mut codepoints = Vec::new();
		let table = face.tables().cmap.expect("Font has no cmap table");

		for subtable in table.subtables.into_iter() {
			if subtable.platform_id != PlatformId::Unicode {
				continue;
			}
			subtable.codepoints(|cp| codepoints.push(cp));
		}

		Ok(FontMetadata {
			family_name,
			style_name,
			codepoints,
		})
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
		assert_eq!(metadata.family_name, "Fira Sans");
		assert_eq!(metadata.style_name, "Regular");
		assert_eq!(metadata.codepoints.len(), 1686);
	}

	#[test]
	fn test_load_noto() {
		const NOTO: &[u8] = include_bytes!("../../testdata/Noto Sans/Noto Sans - Regular.ttf");
		let face = Face::parse(NOTO, 0).unwrap();
		let metadata = FontMetadata::try_from(&face).unwrap();
		assert_eq!(metadata.family_name, "Noto Sans");
		assert_eq!(metadata.style_name, "Regular");
		assert_eq!(metadata.codepoints.len(), 6100);
	}
}
