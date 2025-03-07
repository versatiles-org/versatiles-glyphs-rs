use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
	fs,
	path::{self, Path, PathBuf},
};
use versatiles_glyphs::{
	font::FontManager,
	renderer::RendererPrecise,
	utils::prepare_output_directory,
	writer::{FileWriter, TarWriter, Writer},
};

#[derive(clap::Args, Debug)]
#[command(arg_required_else_help = true)]
/// Scans a directory recursively for font files and convert them into multiple directories of glyphs.
///
/// If a directory contains a "fonts.json" file, it will be used to configure the conversion.
/// A "fonts.json" has the structure: { name: string, sources: string[] }[] where:
///   - name: the name of the font, like "Noto Sans Regular".
///   - sources: the list of font files to merge, relative to the directory.
pub struct Subcommand {
	/// directories to scan for font files.
	#[arg(num_args=1..)]
	input_directories: Vec<PathBuf>,

	/// the output directory where the glyph folders will be saved.
	#[arg(long, short = 'o', conflicts_with = "tar")]
	output_directory: Option<String>,

	/// instead of writing the glyphs to a directory, write them as a tar to stdout.
	#[arg(long, short = 't', conflicts_with = "output_directory")]
	tar: bool,

	/// do not write the font_families.json file.
	#[arg(long)]
	no_families: bool,

	/// do not write the index.json file.
	#[arg(long)]
	no_index: bool,
}

#[derive(Debug, Deserialize)]
struct FontConfig {
	name: String,
	sources: Vec<String>,
}

pub fn run(arguments: &Subcommand) -> Result<()> {
	let mut font_manager = FontManager::default();

	for input_directory in &arguments.input_directories {
		let input_directory = path::absolute(input_directory)?.canonicalize()?;
		eprintln!("Scanning directory: {:?}", input_directory);
		scan(&input_directory, &mut font_manager)?;
	}

	let mut writer: Box<dyn Writer> = if arguments.tar {
		eprintln!("Rendering glyphs as tar to stdout");
		Box::new(TarWriter::new(std::io::stdout()))
	} else {
		let output_directory =
			prepare_output_directory(arguments.output_directory.as_deref().unwrap_or("output"))?;

		eprintln!("Rendering glyphs to directory: {:?}", output_directory);
		Box::new(FileWriter::new(path::absolute(output_directory)?))
	};

	font_manager.render_glyphs(&mut writer, RendererPrecise {})?;
	if !arguments.no_index {
		font_manager.write_index_json(&mut writer)?;
	}
	if !arguments.no_families {
		font_manager.write_families_json(&mut writer)?;
	}

	writer.finish()?;

	Ok(())
}

fn scan(input: &Path, font_manager: &mut FontManager) -> Result<()> {
	if input.is_file() {
		let extension = input.extension().unwrap_or_default().to_str().unwrap();
		if extension == "ttf" || extension == "otf" {
			font_manager.add_path(input)?;
		}
	} else if input.is_dir() {
		let font_file = input.join("fonts.json");
		if font_file.exists() {
			let data =
				fs::read(&font_file).with_context(|| format!("reading font file \"{font_file:?}\""))?;

			let font_configs = serde_json::from_slice::<Vec<FontConfig>>(&data)?;

			for font_config in font_configs {
				font_manager.add_font_with_name(
					&font_config.name,
					&font_config
						.sources
						.iter()
						.map(|source| input.join(source))
						.collect::<Vec<_>>(),
				)?;
			}
		} else {
			for entry in fs::read_dir(input)? {
				scan(&entry?.path(), font_manager)?;
			}
		}
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use versatiles_glyphs::font::FontWrapper;

	use super::*;
	use std::path::PathBuf;

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

		let mut font_manager = FontManager::default();
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
}
