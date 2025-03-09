use crate::{font::FontManager, render::Renderer, utils::prepare_output_directory, writer::Writer};
use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
	fs,
	io::Write,
	path::{self, Path, PathBuf},
};

/// Subcommand arguments for recursively scanning font files.
#[derive(clap::Args, Debug)]
#[command(arg_required_else_help = true)]
/// Recursively scans directories for `.ttf` or `.otf` files and converts them.
///
/// If a directory contains a "fonts.json" file, it will be used to configure the conversion.
/// A "fonts.json" has the structure: { name: string, sources: string[] }[] where:
///   - name: the name of the font, like "Noto Sans Regular".
///   - sources: the list of font files to merge, relative to the directory.
///
/// # Examples
///
/// ```bash
/// versatiles_glyphs recurse -o glyphs my_font_directory
/// versatiles_glyphs recurse -t another_directory
/// ```
pub struct Subcommand {
	/// Directories to scan for font files.
	#[arg(num_args=1..)]
	input_directories: Vec<PathBuf>,

	/// Output directory for glyphs. Mutually exclusive with `tar`.
	#[arg(long, short = 'o', conflicts_with = "tar")]
	output_directory: Option<String>,

	/// Write glyphs as a tar to stdout. Mutually exclusive with `output_directory`.
	#[arg(long, short = 't', conflicts_with = "output_directory")]
	tar: bool,

	/// Skip writing the `font_families.json` file.
	#[arg(long)]
	no_families: bool,

	/// Skip writing the `index.json` file.
	#[arg(long)]
	no_index: bool,

	/// Hidden argument to allow specifying the dummy renderer.
	#[arg(long, hide = true)]
	dummy: bool,

	/// Hidden argument to render glyphs in just a single thread.
	#[arg(long, hide = true)]
	single_thread: bool,
}

/// Describes the structure of a `fonts.json` for merged font sets.
#[derive(Debug, Deserialize)]
struct FontConfig {
	/// Descriptive name for the font or set of merged fonts.
	name: String,
	/// Paths to `.ttf` / `.otf` files, relative to the containing folder.
	sources: Vec<String>,
}

/// Executes the recurse subcommand logic.
///
/// Scans specified directories, reading `fonts.json` if present, and
/// merges fonts into a [`FontManager`]. The glyph data is written
/// either to a directory or stdout tar.
pub fn run(args: &Subcommand, stdout: &mut (impl Write + Send + Sync + 'static)) -> Result<()> {
	let mut font_manager = FontManager::new(!args.single_thread);

	for dir in &args.input_directories {
		let canonical = path::absolute(dir)?.canonicalize()?;
		eprintln!("Scanning directory: {:?}", canonical);
		scan(&canonical, &mut font_manager)?;
	}

	let mut writer = if args.tar {
		eprintln!("Rendering glyphs as tar to stdout.");
		Writer::new_tar(stdout)
	} else {
		let out_dir = prepare_output_directory(args.output_directory.as_deref().unwrap_or("output"))?;
		eprintln!("Rendering glyphs to directory: {:?}", out_dir);
		Writer::new_file(path::absolute(out_dir)?)
	};

	let renderer = Renderer::new(args.dummy);

	font_manager.render_glyphs(&mut writer, &renderer)?;
	if !args.no_index {
		font_manager.write_index_json(&mut writer)?;
	}
	if !args.no_families {
		font_manager.write_families_json(&mut writer)?;
	}

	writer.finish()?;

	Ok(())
}

/// Recursively scans directories and adds matching font files to the [`FontManager`].
fn scan(path: &Path, font_manager: &mut FontManager) -> Result<()> {
	if path.is_file() {
		let extension = path.extension().unwrap_or_default().to_str().unwrap();
		if extension == "ttf" || extension == "otf" {
			font_manager.add_path(path)?;
		}
	} else if path.is_dir() {
		let font_file = path.join("fonts.json");
		if font_file.exists() {
			let data =
				fs::read(&font_file).with_context(|| format!("Failed to read {:?}", font_file))?;
			let configs = serde_json::from_slice::<Vec<FontConfig>>(&data)?;

			for c in configs {
				font_manager.add_font_with_name(
					&c.name,
					&c.sources
						.iter()
						.map(|src| path.join(src))
						.collect::<Vec<_>>(),
				)?;
			}
		} else {
			for entry in fs::read_dir(path)? {
				scan(&entry?.path(), font_manager)?;
			}
		}
	}
	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::font::FontWrapper;

	fn get_names(font: &FontWrapper) -> Vec<String> {
		let mut names = font
			.files
			.iter()
			.map(|f| f.metadata.name.clone())
			.collect::<Vec<_>>();
		names.sort_unstable();
		names
	}

	#[test]
	fn test_scan() -> Result<()> {
		let dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata");
		let mut font_manager = FontManager::new(false);
		scan(&dir_path, &mut font_manager)?;

		let mut keys = font_manager.fonts.keys().collect::<Vec<_>>();
		keys.sort_unstable();
		assert_eq!(keys, ["fira_sans_regular", "noto_sans_regular"]);

		assert_eq!(
			get_names(font_manager.fonts.get("noto_sans_regular").unwrap()),
			[
				"Noto Sans",
				"Noto Sans Arabic",
				"Noto Sans Armenian",
				"Noto Sans Balinese",
				"Noto Sans Bengali",
				"Noto Sans Devanagari",
				"Noto Sans Ethiopic",
				"Noto Sans Georgian",
				"Noto Sans Gujarati",
				"Noto Sans Gurmukhi",
				"Noto Sans Hebrew",
				"Noto Sans JP",
				"Noto Sans Javanese",
				"Noto Sans KR",
				"Noto Sans Kannada",
				"Noto Sans Khmer",
				"Noto Sans Lao",
				"Noto Sans Myanmar",
				"Noto Sans Oriya",
				"Noto Sans SC",
				"Noto Sans Sinhala",
				"Noto Sans Tamil",
				"Noto Sans Thai"
			]
		);

		assert_eq!(
			get_names(font_manager.fonts.get("fira_sans_regular").unwrap()),
			["Fira Sans"]
		);

		Ok(())
	}

	#[test]
	fn test_run_with_tar_to_stdout() -> Result<()> {
		// Pretend we have multiple directories, but they actually reference the same testdata dir.
		let args = Subcommand {
			input_directories: vec![PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata")],
			output_directory: None,
			tar: true,
			no_families: false,
			no_index: false,
			dummy: true,
			single_thread: false,
		};

		let mut stdout = Vec::<u8>::new();
		run(&args, &mut stdout)?;

		assert_eq!(stdout.len(), 39496192);

		Ok(())
	}
	/*
	#[test]
	fn test_run_dummy_renderer_no_index_or_families() -> Result<()> {
		// This tests skipping "font_families.json" and "index.json"
		// while using the dummy renderer and writing to a "dummy" writer
		// by stubbing out the final writer creation.
		let dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata");
		let mut font_manager = FontManager::new(false);
		scan(&dir_path, &mut font_manager)?;

		// Instead of the full run() logic, we replicate the key steps with a dummy writer:
		let mut writer: Box<dyn Writer> = Box::new(DummyWriter::default());
		let renderer = Renderer::new(true); // --dummy
		font_manager.render_glyphs(&mut writer, &renderer)?;
		// no_index -> skip
		// no_families -> skip
		writer.finish()?;

		// The dummy writer won't contain index or families JSON files.
		#[cfg(test)]
		{
			let logs = writer.get_inner().unwrap();
			// We expect only glyph data. Typically includes .pbf, etc.
			assert!(!logs.is_empty());
			// We specifically expect not to see index.json or font_families.json
			let index_or_families = logs
				.iter()
				.any(|line| line.contains("index.json") || line.contains("font_families.json"));
			assert!(!index_or_families);
		}

		Ok(())
	}

	#[test]
	fn test_run_single_thread_precise_renderer() -> Result<()> {
		// Similar to the above test, but we confirm single_thread mode is set,
		// and we use the precise renderer. We won't rely on the run() function
		// because that would write to real files or stdout. We'll replicate steps.

		let dir_path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata");

		// Force single-thread
		// There's no direct "check" for single-thread usage except that
		// FontManager's parallel usage is turned off. We'll just pass "false" to new.
		let mut font_manager = FontManager::new(false);
		scan(&dir_path, &mut font_manager)?;

		// use a dummy writer for testing
		let mut writer: Box<dyn Writer> = Box::new(DummyWriter::default());
		let renderer = Renderer::new_precise(); // the "precise" version
		font_manager.render_glyphs(&mut writer, &renderer)?;
		writer.finish()?;

		// The dummy writer receives .pbf files, so let's ensure there's some output
		#[cfg(test)]
		{
			let logs = writer.get_inner().unwrap();
			assert!(!logs.is_empty());
			let pbf_files = logs.iter().any(|line| line.contains(".pbf"));
			assert!(pbf_files);
		}

		Ok(())
	}

	#[test]
	fn test_non_existing_directory_is_noop() -> Result<()> {
		// Attempt scanning a non-existing directory. It should do nothing, no error thrown.
		let non_existent = PathBuf::from("does-not-exist-xyz");
		let mut font_manager = FontManager::new(false);
		// Should return Ok and not panic.
		scan(&non_existent, &mut font_manager)?;
		// The font manager is still empty.
		assert!(font_manager.fonts.is_empty());
		Ok(())
	}

	#[test]
	fn test_invalid_fonts_json() {
		// Attempt scanning a directory with a malformed fonts.json.
		// We'll place a purposely invalid JSON in a temp directory to confirm error handling.
		let tmp_dir = tempfile::tempdir().unwrap();
		let tmp_fonts = tmp_dir.path().join("fonts.json");
		fs::write(&tmp_fonts, b"{ invalid ").unwrap(); // incomplete JSON

		let mut font_manager = FontManager::new(false);
		let result = scan(tmp_dir.path(), &mut font_manager);
		// Expect a serde error
		assert!(result.is_err());
		let err_msg = format!("{:?}", result.err().unwrap());
		assert!(err_msg.contains("Failed to read"));
	}
	 */
}
