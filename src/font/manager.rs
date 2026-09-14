use super::index_files::{build_font_families_json, build_index_json};
use crate::{
	font::{FontFileEntry, FontWrapper, GlyphBlock},
	render::Renderer,
	utils::get_progress_bar,
	writer::Writer,
};
use anyhow::Result;
use rayon::iter::{IntoParallelIterator, ParallelIterator};
use regex_lite::Regex;
use std::{
	collections::{btree_map::Entry, hash_map, BTreeMap, HashMap},
	path::{Path, PathBuf},
	sync::OnceLock,
};

/// Manages a collection of fonts and provides methods to render glyphs
/// and write metadata (index/families) files.
pub struct FontManager<'a> {
	/// Mapping from a font identifier to a [`FontWrapper`].
	///
	/// Ordered, so fonts are always rendered and written sorted by identifier.
	pub fonts: BTreeMap<String, FontWrapper<'a>>,
	/// Whether to parallelize rendering operations.
	pub parallel: bool,
	/// Whether to write a range that is identical to an already written range
	/// as a hardlink instead of rendering it again. See [`Self::render_glyphs`].
	pub link_duplicates: bool,
}

/// What to write for a single glyph block.
enum BlockOutput {
	/// The rendered PBF data.
	Data(Vec<u8>),
	/// The path of an identical, already written block to link to.
	Link(String),
}

impl<'a> FontManager<'a> {
	/// Creates a new `FontManager` with the specified parallel rendering setting.
	pub fn new(parallel: bool) -> Self {
		Self {
			fonts: BTreeMap::new(),
			parallel,
			link_duplicates: false,
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
	/// The output order is deterministic: fonts are written sorted by identifier,
	/// each followed by its blocks in ascending range order. The blocks of a font
	/// (and the glyphs within each block) are rendered in parallel with `rayon`
	/// (if enabled), buffered, and then written in order, so at most one font's
	/// glyph data is held in memory.
	///
	/// If [`Self::link_duplicates`] is set, a non-empty block whose glyphs all come from
	/// the same source files as an already written block (see [`GlyphBlock::source_key`])
	/// is not rendered, but written as a hardlink to that earlier block. This happens
	/// when several fonts share fallback files, e.g. an italic face that falls back
	/// to the upright CJK fonts.
	pub fn render_glyphs(&'a self, writer: &mut Writer, renderer: &Renderer) -> Result<()> {
		// Collect all blocks from every font.
		let fonts = self
			.fonts
			.iter()
			.map(|(name, font)| (name, font.get_blocks()))
			.collect::<Vec<_>>();

		// Progress bar across all glyph blocks.
		let total_glyphs = fonts
			.iter()
			.flat_map(|(_, blocks)| blocks)
			.map(|block| block.len() as u64)
			.sum();
		let progress = get_progress_bar(total_glyphs);

		// Path of the first block written for each source key.
		let mut written_paths = HashMap::<_, String>::new();

		for (name, blocks) in &fonts {
			writer.write_directory(&format!("{name}/"))?;

			// Pair each block with the path of an identical block written before, if any.
			let tasks = blocks
				.iter()
				.map(|block| {
					let path = format!("{name}/{}", block.filename());
					let link = match block.source_key().filter(|_| self.link_duplicates) {
						None => None,
						Some(key) => match written_paths.entry(key) {
							hash_map::Entry::Occupied(e) => Some(e.get().clone()),
							hash_map::Entry::Vacant(e) => {
								e.insert(path.clone());
								None
							}
						},
					};
					(block, path, link)
				})
				.collect::<Vec<_>>();

			let process = |(block, path, link): (&GlyphBlock, String, Option<String>)| -> Result<_> {
				let output = match link {
					Some(target) => BlockOutput::Link(target),
					None => BlockOutput::Data(block.render(renderer, self.parallel)?),
				};
				progress.inc(block.len() as u64);
				Ok((path, output))
			};

			let outputs = if self.parallel {
				tasks
					.into_par_iter()
					.map(process)
					.collect::<Result<Vec<_>>>()?
			} else {
				tasks.into_iter().map(process).collect::<Result<Vec<_>>>()?
			};

			for (path, output) in outputs {
				match output {
					BlockOutput::Data(data) => writer.write_file(&path, &data)?,
					BlockOutput::Link(target) => writer.write_link(&path, &target)?,
				}
			}
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
		// is now emitted as an empty placeholder rather than being absent. It's just an
		// empty fontstack (`0a 00`).
		assert_eq!(size_of("noto_sans_regular", 3840), 2);
		Ok(())
	}

	#[test]
	fn test_render_glyphs_write_order() -> Result<()> {
		let mut manager = FontManager::new(true);
		manager.add_paths(&get_test_paths())?;

		let mut writer = Writer::new_dummy();
		manager.render_glyphs(&mut writer, &Renderer::new_dummy())?;

		// Strip the " (size)" suffix, keeping the order in which entries were written.
		let written = writer
			.get_inner()
			.unwrap()
			.iter()
			.map(|e| e.split(" (").next().unwrap().to_string())
			.collect::<Vec<_>>();

		// Fonts sorted by id; each directory followed by its ranges in ascending order.
		let mut expected = Vec::new();
		for font in ["fira_sans_regular", "noto_sans_regular"] {
			expected.push(format!("{font}/"));
			for i in 0..256 {
				let start = i * crate::font::GLYPH_BLOCK_SIZE;
				let end = start + crate::font::GLYPH_BLOCK_SIZE - 1;
				expected.push(format!("{font}/{start}-{end}.pbf"));
			}
		}
		assert_eq!(written, expected);
		Ok(())
	}

	#[test]
	fn test_render_glyphs_is_reproducible() -> Result<()> {
		let render = || -> Result<Vec<u8>> {
			let mut manager = FontManager::new(true);
			manager.add_paths(&get_test_paths())?;

			let mut output = Vec::<u8>::new();
			let mut writer = Writer::new_tar(&mut output);
			manager.render_glyphs(&mut writer, &Renderer::new_dummy())?;
			manager.write_index_json(&mut writer)?;
			manager.write_families_json(&mut writer)?;
			writer.finish()?;
			drop(writer);
			Ok(output)
		};

		let first = render()?;
		let second = render()?;
		assert!(first == second, "two builds produced different bytes");
		Ok(())
	}

	/// Two faces that share `Noto Sans Arabic` as fallback, like an italic face
	/// falling back to upright fonts.
	fn get_manager_with_shared_fallback<'a>(link_duplicates: bool) -> Result<FontManager<'a>> {
		let d = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata");
		let arabic = d.join("Noto Sans/Noto Sans Arabic - Regular.ttf");
		let mut manager = FontManager::new(true);
		manager.link_duplicates = link_duplicates;
		manager.add_font_with_name(
			"Face A",
			&[d.join("Noto Sans/Noto Sans - Regular.ttf"), arabic.clone()],
		)?;
		manager.add_font_with_name("Face B", &[d.join("Fira Sans - Regular.ttf"), arabic])?;
		Ok(manager)
	}

	fn render_tar(manager: &FontManager) -> Result<Vec<u8>> {
		let mut output = Vec::<u8>::new();
		let mut writer = Writer::new_tar(&mut output);
		manager.render_glyphs(&mut writer, &Renderer::new_dummy())?;
		writer.finish()?;
		drop(writer);
		Ok(output)
	}

	#[test]
	fn test_render_glyphs_links_duplicates() -> Result<()> {
		let manager = get_manager_with_shared_fallback(true)?;
		let mut writer = Writer::new_dummy();
		manager.render_glyphs(&mut writer, &Renderer::new_dummy())?;

		let entries = writer.get_inner().unwrap();
		let links = entries
			.iter()
			.filter(|e| e.contains(" -> "))
			.collect::<Vec<_>>();

		// Ranges only covered by the shared Arabic font are linked from B to A.
		assert!(links.contains(&&"face_b/1536-1791.pbf -> face_a/1536-1791.pbf".to_string()));
		// Every link points from the later face to the same range of the earlier one.
		for link in &links {
			let (from, to) = link.split_once(" -> ").unwrap();
			assert!(from.starts_with("face_b/"), "{link}");
			assert_eq!(from.replace("face_b/", "face_a/"), to, "{link}");
		}
		// Ranges with glyphs from different fonts, and empty ranges, are written as files.
		assert!(entries.iter().any(|e| e.starts_with("face_b/0-255.pbf (")));
		assert!(entries.contains(&"face_b/3840-4095.pbf (2)".to_string()));
		Ok(())
	}

	#[test]
	fn test_render_glyphs_without_link_duplicates_writes_files() -> Result<()> {
		let manager = get_manager_with_shared_fallback(false)?;
		let mut writer = Writer::new_dummy();
		manager.render_glyphs(&mut writer, &Renderer::new_dummy())?;
		assert!(!writer
			.get_inner()
			.unwrap()
			.iter()
			.any(|e| e.contains(" -> ")));
		Ok(())
	}

	#[test]
	fn test_linked_tar_extracts_to_identical_files() -> Result<()> {
		use std::{collections::BTreeMap, fs, path::Path};

		fn read_tree(dir: &Path) -> BTreeMap<String, Vec<u8>> {
			let mut files = BTreeMap::new();
			for font in fs::read_dir(dir).unwrap() {
				for file in fs::read_dir(font.unwrap().path()).unwrap() {
					let path = file.unwrap().path();
					let key = path
						.strip_prefix(dir)
						.unwrap()
						.to_string_lossy()
						.to_string();
					files.insert(key, fs::read(&path).unwrap());
				}
			}
			files
		}

		let plain = render_tar(&get_manager_with_shared_fallback(false)?)?;
		let linked = render_tar(&get_manager_with_shared_fallback(true)?)?;
		assert!(linked.len() < plain.len());

		let plain_dir = tempfile::tempdir()?;
		tar::Archive::new(&plain[..]).unpack(plain_dir.path())?;
		let expected = read_tree(plain_dir.path());
		assert_eq!(expected.len(), 512);

		// Extracting with the `tar` crate recreates every linked file.
		let linked_dir = tempfile::tempdir()?;
		tar::Archive::new(&linked[..]).unpack(linked_dir.path())?;
		assert!(read_tree(linked_dir.path()) == expected);

		// So does the system `tar` (GNU or BSD), if available.
		let tar_path = linked_dir.path().join("linked.tar");
		fs::write(&tar_path, &linked)?;
		let system_dir = tempfile::tempdir()?;
		let status = std::process::Command::new("tar")
			.arg("-xf")
			.arg(&tar_path)
			.arg("-C")
			.arg(system_dir.path())
			.status();
		if let Ok(status) = status {
			assert!(status.success());
			assert!(read_tree(system_dir.path()) == expected);
		}
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
