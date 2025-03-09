use super::index_files::{build_font_families_json, build_index_json};
use crate::{
	font::{FontFileEntry, FontWrapper, GlyphBlock},
	render::Renderer,
	utils::get_progress_bar,
	writer::Writer,
};
use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex_lite::Regex;
use std::{
	collections::{hash_map::Entry, HashMap},
	path::{Path, PathBuf},
	sync::Mutex,
};

/// Manages a collection of fonts and provides methods to render glyphs
/// and write metadata (index/families) files.
pub struct FontManager<'a> {
	/// Mapping from a font identifier to a [`FontWrapper`].
	pub fonts: HashMap<String, FontWrapper<'a>>,
	pub parallel: bool,
}

impl<'a> FontManager<'a> {
	pub fn new(parallel: bool) -> Self {
		Self {
			fonts: HashMap::new(),
			parallel,
		}
	}

	/// Adds a single font file to the manager by path.
	///
	/// The font name is normalized to form a key used in [`self.fonts`].
	/// If the key already exists, the file is appended to that font.
	pub fn add_path(&mut self, path: &Path) -> Result<()> {
		let file_data = std::fs::read(path)?;
		let file = FontFileEntry::new(file_data)?;
		let id = name_to_id(&file.metadata.generate_name());

		match self.fonts.entry(id) {
			Entry::Vacant(e) => {
				e.insert(FontWrapper::from(file));
			}
			Entry::Occupied(mut e) => {
				e.get_mut().add_file(file);
			}
		}
		Ok(())
	}

	/// Adds multiple font files to the manager.
	pub fn add_paths(&mut self, paths: &[PathBuf]) -> Result<()> {
		for p in paths {
			self.add_path(p)?;
		}
		Ok(())
	}

	/// Adds multiple sources for a single named font family.
	///
	/// Useful for merging multiple `.ttf` files under one key.
	pub fn add_font_with_name(&mut self, name: &str, sources: &[PathBuf]) -> Result<()> {
		let id = name_to_id(name);
		self
			.fonts
			.entry(id)
			.and_modify(|f| f.add_paths(sources).unwrap())
			.or_insert_with(|| FontWrapper::try_from(sources).unwrap());
		Ok(())
	}

	/// Renders glyphs from all managed fonts via the provided renderer,
	/// writing each glyph block to the supplied writer.
	///
	/// Rendering is parallelized with `rayon` for performance.
	pub fn render_glyphs(&'a self, writer: &mut Box<dyn Writer>, renderer: &Renderer) -> Result<()> {
		struct Todo<'block> {
			name: String,
			block: GlyphBlock<'block>,
		}

		// Collect all blocks from every font.
		let mut tasks = Vec::new();
		for (name, font) in &self.fonts {
			writer.write_directory(&format!("{}/", name))?;
			for block in font.get_blocks() {
				tasks.push(Todo {
					name: name.clone(),
					block,
				});
			}
		}

		// Progress bar across all glyph blocks.
		let total_glyphs = tasks.iter().map(|t| t.block.len() as u64).sum();
		let progress = get_progress_bar(total_glyphs);
		let writer_mutex = Mutex::new(writer);

		let op = |todo: &Todo| {
			let file_name = format!("{}/{}", todo.name, todo.block.filename());
			let data = todo.block.render(todo.name.clone(), renderer).unwrap();

			// Lock the writer and write out the rendered data.
			writer_mutex
				.lock()
				.unwrap()
				.write_file(&file_name, &data)
				.unwrap();

			progress.inc(todo.block.len() as u64);
		};

		if self.parallel {
			tasks.par_iter().for_each(op);
		} else {
			tasks.iter().for_each(op);
		}

		progress.finish();
		Ok(())
	}

	/// Writes an index of all font IDs to `index.json`.
	pub fn write_index_json(&self, writer: &mut Box<dyn Writer>) -> Result<()> {
		let content = build_index_json(self.fonts.keys())?;
		writer.write_file("index.json", &content)
	}

	/// Writes a list of font families and their styles/weights to `font_families.json`.
	pub fn write_families_json(&self, writer: &mut Box<dyn Writer>) -> Result<()> {
		let content = build_font_families_json(self.fonts.iter())?;
		writer.write_file("font_families.json", &content)
	}
}

/// Normalizes a font name into a lowercase, underscore-delimited string.
fn name_to_id(name: &str) -> String {
	let mut lower = name.to_lowercase();
	let re = Regex::new(r"[-_\s]+").unwrap();
	lower = re.replace_all(&lower, " ").trim().to_string();
	lower.replace(' ', "_")
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::writer::dummy::DummyWriter;

	fn get_test_paths() -> Vec<PathBuf> {
		let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata");
		vec![
			d.join("Fira Sans - Regular.ttf"),
			d.join("Noto Sans/Noto Sans - Regular.ttf"),
			d.join("Noto Sans/Noto Sans Arabic - Regular.ttf"),
			d.join("Noto Sans/Noto Sans Tamil - Regular.ttf"),
		]
	}

	#[test]
	fn test_render_glyphs() -> Result<()> {
		let mut manager = FontManager::new(false);
		manager.add_paths(&get_test_paths())?;

		assert_eq!(manager.fonts.len(), 2);
		let mut writer: Box<dyn Writer> = Box::new(DummyWriter::default());
		manager.render_glyphs(&mut writer, &Renderer::new_dummy())?;

		let mut files = writer.get_inner().unwrap().to_vec();
		files.sort_unstable();

		assert_eq!(
			files,
			[
				"fira_sans_regular/",
				"fira_sans_regular/0-255.pbf (79666)",
				"fira_sans_regular/1024-1279.pbf (117144)",
				"fira_sans_regular/11264-11519.pbf (3527)",
				"fira_sans_regular/1280-1535.pbf (26175)",
				"fira_sans_regular/256-511.pbf (130212)",
				"fira_sans_regular/3584-3839.pbf (592)",
				"fira_sans_regular/42752-43007.pbf (5790)",
				"fira_sans_regular/43776-44031.pbf (487)",
				"fira_sans_regular/512-767.pbf (92229)",
				"fira_sans_regular/64256-64511.pbf (1007)",
				"fira_sans_regular/65024-65279.pbf (50)",
				"fira_sans_regular/7424-7679.pbf (7196)",
				"fira_sans_regular/768-1023.pbf (63380)",
				"fira_sans_regular/7680-7935.pbf (86554)",
				"fira_sans_regular/7936-8191.pbf (125259)",
				"fira_sans_regular/8192-8447.pbf (20252)",
				"fira_sans_regular/8448-8703.pbf (17542)",
				"fira_sans_regular/8704-8959.pbf (6396)",
				"fira_sans_regular/8960-9215.pbf (4375)",
				"fira_sans_regular/9472-9727.pbf (876)",
				"noto_sans_regular/",
				"noto_sans_regular/0-255.pbf (83352)",
				"noto_sans_regular/1024-1279.pbf (134023)",
				"noto_sans_regular/11264-11519.pbf (15601)",
				"noto_sans_regular/11520-11775.pbf (6086)",
				"noto_sans_regular/11776-12031.pbf (31828)",
				"noto_sans_regular/122624-122879.pbf (16407)",
				"noto_sans_regular/1280-1535.pbf (29050)",
				"noto_sans_regular/1536-1791.pbf (120473)",
				"noto_sans_regular/1792-2047.pbf (32352)",
				"noto_sans_regular/2048-2303.pbf (29603)",
				"noto_sans_regular/2304-2559.pbf (59478)",
				"noto_sans_regular/256-511.pbf (137913)",
				"noto_sans_regular/2816-3071.pbf (54726)",
				"noto_sans_regular/4096-4351.pbf (477)",
				"noto_sans_regular/42496-42751.pbf (50378)",
				"noto_sans_regular/42752-43007.pbf (107430)",
				"noto_sans_regular/43008-43263.pbf (636)",
				"noto_sans_regular/43264-43519.pbf (253)",
				"noto_sans_regular/43776-44031.pbf (27365)",
				"noto_sans_regular/512-767.pbf (103422)",
				"noto_sans_regular/64256-64511.pbf (89260)",
				"noto_sans_regular/64512-64767.pbf (214778)",
				"noto_sans_regular/64768-65023.pbf (245037)",
				"noto_sans_regular/65024-65279.pbf (73110)",
				"noto_sans_regular/65280-65535.pbf (1786)",
				"noto_sans_regular/6656-6911.pbf (5828)",
				"noto_sans_regular/67328-67583.pbf (16379)",
				"noto_sans_regular/70400-70655.pbf (801)",
				"noto_sans_regular/7168-7423.pbf (4457)",
				"noto_sans_regular/7424-7679.pbf (78090)",
				"noto_sans_regular/768-1023.pbf (77467)",
				"noto_sans_regular/7680-7935.pbf (145644)",
				"noto_sans_regular/7936-8191.pbf (137370)",
				"noto_sans_regular/8192-8447.pbf (58213)",
				"noto_sans_regular/8448-8703.pbf (55808)",
				"noto_sans_regular/8704-8959.pbf (168)",
				"noto_sans_regular/9472-9727.pbf (394)"
			]
		);
		Ok(())
	}

	#[test]
	fn test_write_families_json() -> Result<()> {
		let mut manager = FontManager::new(false);
		manager.add_paths(&get_test_paths())?;

		assert_eq!(manager.fonts.len(), 2);
		let mut writer: Box<dyn Writer> = Box::new(DummyWriter::default());
		manager.write_families_json(&mut writer)?;

		let mut files = writer.get_inner().unwrap().to_vec();
		files.sort_unstable();

		assert_eq!(files.len(), 1);
		assert_eq!(
			&files[0][0..64],
			"font_families.json: [{\"name\": \"Fira Sans\",\"faces\": [{\"id\": \"fira"
		);
		Ok(())
	}

	#[test]
	fn test_write_index_json() -> Result<()> {
		let mut manager = FontManager::new(false);
		manager.add_paths(&get_test_paths())?;

		assert_eq!(manager.fonts.len(), 2);
		let mut writer: Box<dyn Writer> = Box::new(DummyWriter::default());
		manager.write_index_json(&mut writer)?;

		let mut files = writer.get_inner().unwrap().to_vec();
		files.sort_unstable();

		assert_eq!(
			files,
			["index.json: [\"fira_sans_regular\",\"noto_sans_regular\"]"]
		);
		Ok(())
	}
}
