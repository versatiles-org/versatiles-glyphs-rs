use super::index_files::{build_font_families_json, build_index_json};
use crate::{
	font::{FontFileEntry, FontWrapper, GlyphBlock},
	render::Renderer,
	utils::get_progress_bar,
	writer::Writer,
};
use anyhow::{anyhow, Result};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex_lite::Regex;
use std::{
	collections::{hash_map::Entry, HashMap},
	path::{Path, PathBuf},
	sync::{Mutex, OnceLock},
};

/// Manages a collection of fonts and provides methods to render glyphs
/// and write metadata (index/families) files.
pub struct FontManager<'a> {
	/// Mapping from a font identifier to a [`FontWrapper`].
	pub fonts: HashMap<String, FontWrapper<'a>>,
	/// Whether to parallelize rendering operations.
	pub parallel: bool,
}

impl<'a> FontManager<'a> {
	/// Creates a new `FontManager` with the specified parallel rendering setting.
	pub fn new(parallel: bool) -> Self {
		Self {
			fonts: HashMap::new(),
			parallel,
		}
	}

	/// Adds a single font file to the manager by path.
	///
	/// The font name is normalized to form a key used in [`Self::fonts`].
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
		match self.fonts.entry(id) {
			Entry::Occupied(mut e) => e.get_mut().add_paths(sources)?,
			Entry::Vacant(e) => {
				e.insert(FontWrapper::try_from(sources)?);
			}
		}
		Ok(())
	}

	/// Renders glyphs from all managed fonts via the provided renderer,
	/// writing each glyph block to the supplied writer.
	///
	/// Rendering is parallelized with `rayon` for performance.
	pub fn render_glyphs(&'a self, writer: &mut Writer, renderer: &Renderer) -> Result<()> {
		struct Todo<'block> {
			name: String,
			block: GlyphBlock<'block>,
		}

		// Collect all blocks from every font.
		let mut tasks = Vec::new();
		for (name, font) in &self.fonts {
			writer.write_directory(&format!("{name}/"))?;
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

		let op = |todo: &Todo| -> Result<()> {
			let file_name = format!("{}/{}", todo.name, todo.block.filename());
			let data = todo.block.render(todo.name.clone(), renderer)?;

			writer_mutex
				.lock()
				.map_err(|_| anyhow!("writer mutex poisoned"))?
				.write_file(&file_name, &data)?;

			progress.inc(todo.block.len() as u64);
			Ok(())
		};

		if self.parallel {
			tasks.par_iter().try_for_each(op)?;
		} else {
			tasks.iter().try_for_each(op)?;
		}

		progress.finish();
		Ok(())
	}

	/// Writes an index of all font IDs to `index.json`.
	pub fn write_index_json(&self, writer: &mut Writer) -> Result<()> {
		let content = build_index_json(self.fonts.keys())?;
		writer.write_file("index.json", &content)
	}

	/// Writes a list of font families and their styles/weights to `font_families.json`.
	pub fn write_families_json(&self, writer: &mut Writer) -> Result<()> {
		let content = build_font_families_json(self.fonts.iter())?;
		writer.write_file("font_families.json", &content)
	}
}

/// Normalizes a font name into a lowercase, underscore-delimited string.
fn name_to_id(name: &str) -> String {
	static RE: OnceLock<Regex> = OnceLock::new();
	let re = RE.get_or_init(|| Regex::new(r"[-_\s]+").expect("valid regex"));
	let lower = name.to_lowercase();
	let collapsed = re.replace_all(&lower, " ").trim().to_string();
	collapsed.replace(' ', "_")
}

#[cfg(test)]
mod tests {
	use super::*;

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
		let mut writer = Writer::new_dummy();
		manager.render_glyphs(&mut writer, &Renderer::new_dummy())?;

		let mut files = writer.get_inner().unwrap().to_vec();
		files.sort_unstable();

		// Both logical fonts get a directory entry plus one `.pbf` per BMP range.
		assert!(files.contains(&"fira_sans_regular/".to_string()));
		assert!(files.contains(&"noto_sans_regular/".to_string()));

		// Parse each entry into (font, range_start, size). Directory entries have no `.pbf`.
		let parse = |entry: &str| -> Option<(String, u32, usize)> {
			let (path, rest) = entry.split_once(".pbf (")?;
			let (font, range) = path.split_once('/')?;
			let start = range.split('-').next()?.parse::<u32>().ok()?;
			let size = rest.trim_end_matches(')').parse::<usize>().ok()?;
			Some((font.to_string(), start, size))
		};
		let glyphs = files.iter().filter_map(|e| parse(e)).collect::<Vec<_>>();

		for font in ["fira_sans_regular", "noto_sans_regular"] {
			let ranges = glyphs
				.iter()
				.filter(|(f, ..)| f == font)
				.map(|&(_, start, _)| start)
				.collect::<Vec<_>>();

			// Every BMP range (0-255 … 65280-65535) is present exactly once, and nothing
			// beyond the BMP: emitting empty ranges is what stops MapLibre's 404 warnings,
			// and the BMP cap drops astral ranges the clients would never request.
			assert_eq!(ranges.len(), 256, "{font} should emit all 256 BMP ranges");
			let mut sorted = ranges.clone();
			sorted.sort_unstable();
			sorted.dedup();
			assert_eq!(sorted.len(), 256, "{font} ranges must be unique");
			assert_eq!(*sorted.first().unwrap(), 0);
			assert_eq!(*sorted.last().unwrap(), 65280);
			assert!(sorted
				.iter()
				.all(|s| s % crate::font::GLYPH_BLOCK_SIZE == 0));
		}

		// A range the font covers is a substantial file; a gap range is a tiny empty pbf.
		let size_of = |font: &str, start: u32| {
			glyphs
				.iter()
				.find(|&&(ref f, s, _)| f == font && s == start)
				.map(|&(_, _, size)| size)
				.unwrap()
		};
		assert!(size_of("noto_sans_regular", 256) > 1000);
		// U+0F00–0FFF (Tibetan, range 3840-4095) — the exact 404 reported by MapLibre —
		// is now emitted as an empty placeholder rather than being absent.
		assert!(size_of("noto_sans_regular", 3840) < 100);
		Ok(())
	}

	#[test]
	fn test_write_families_json() -> Result<()> {
		let mut manager = FontManager::new(false);
		manager.add_paths(&get_test_paths())?;

		assert_eq!(manager.fonts.len(), 2);
		let mut writer = Writer::new_dummy();
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
		let mut writer = Writer::new_dummy();
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
