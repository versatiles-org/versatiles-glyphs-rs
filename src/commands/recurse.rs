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
		eprintln!("Scanning directory: {canonical:?}");
		scan(&canonical, &mut font_manager)?;
	}

	let mut writer = if args.tar {
		eprintln!("Rendering glyphs as tar to stdout.");
		Writer::new_tar(stdout)
	} else {
		let out_dir = prepare_output_directory(args.output_directory.as_deref().unwrap_or("output"))?;
		eprintln!("Rendering glyphs to directory: {out_dir:?}");
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
				fs::read(&font_file).with_context(|| format!("Failed to read {font_file:?}"))?;
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

	fn get_tar_entries(data: &[u8]) -> Vec<String> {
		let mut tar = tar::Archive::new(data);
		let mut entries = tar
			.entries()
			.unwrap()
			.filter_map(|e| {
				let e = e.unwrap();
				if e.size() >= 32 && e.size() <= 34 {
					return None;
				}
				Some(format!("{:?}: {}", e.path().unwrap(), e.size()))
			})
			.collect::<Vec<_>>();
		entries.sort_unstable();
		entries
	}

	#[test]
	fn test_run_with_tar_to_stdout() -> Result<()> {
		// Pretend we have multiple directories, but they actually reference the same testdata dir.
		let args = Subcommand {
			input_directories: vec![
				PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/Fira Sans - Regular.ttf")
			],
			output_directory: None,
			tar: true,
			no_families: false,
			no_index: false,
			dummy: true,
			single_thread: false,
		};

		let mut stdout = Vec::<u8>::new();
		run(&args, &mut stdout)?;

		assert_eq!(
			get_tar_entries(&stdout),
			[
				"\"fira_sans_regular/\": 0",
				"\"fira_sans_regular/0-255.pbf\": 80022",
				"\"fira_sans_regular/1024-1279.pbf\": 118037",
				"\"fira_sans_regular/11264-11519.pbf\": 3579",
				"\"fira_sans_regular/1280-1535.pbf\": 26296",
				"\"fira_sans_regular/256-511.pbf\": 130750",
				"\"fira_sans_regular/3584-3839.pbf\": 592",
				"\"fira_sans_regular/42752-43007.pbf\": 5761",
				"\"fira_sans_regular/43776-44031.pbf\": 487",
				"\"fira_sans_regular/512-767.pbf\": 92634",
				"\"fira_sans_regular/64256-64511.pbf\": 1032",
				"\"fira_sans_regular/65024-65279.pbf\": 50",
				"\"fira_sans_regular/7424-7679.pbf\": 7260",
				"\"fira_sans_regular/768-1023.pbf\": 63760",
				"\"fira_sans_regular/7680-7935.pbf\": 87078",
				"\"fira_sans_regular/7936-8191.pbf\": 124520",
				"\"fira_sans_regular/8192-8447.pbf\": 20301",
				"\"fira_sans_regular/8448-8703.pbf\": 17395",
				"\"fira_sans_regular/8704-8959.pbf\": 6511",
				"\"fira_sans_regular/8960-9215.pbf\": 4375",
				"\"fira_sans_regular/9472-9727.pbf\": 853",
				"\"font_families.json\": 365",
				"\"index.json\": 25"
			]
		);

		Ok(())
	}
}
