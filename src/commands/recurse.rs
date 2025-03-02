use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
	fs,
	path::{self, Path, PathBuf},
};
use versatiles_glyphs::font::FontRenderer;
// use std::{fs, path};
// use versatiles_glyphs::font::FontManager;

#[derive(clap::Args, Debug)]
#[command(arg_required_else_help = true)]
/// Scans a directory for font files and convert them into multiple directories of glyphs.
///
/// If a directory contains a "fonts.json" file, it will be used to configure the conversion.
/// A "fonts.json" has the structure: { name: string, sources: string[] }[] where:
///   - name: the name of the font.
///   - sources: the list of font files to merge, relative to the directory.
pub struct Subcommand {
	/// directory to scan for font files.
	#[arg()]
	input_directory: String,

	/// the output directory where the glyph folders will be saved.
	#[arg(long, short = 'o', default_value = "output")]
	output_directory: String,
}

#[derive(Debug, Deserialize)]
struct FontConfig {
	name: String,
	sources: Vec<String>,
}

pub fn run(arguments: &Subcommand) -> Result<()> {
	let input_directory = path::absolute(&arguments.input_directory)?;
	let output_directory = path::absolute(&arguments.output_directory)?;

	if output_directory.exists() {
		fs::remove_dir_all(&output_directory)
			.with_context(|| format!("removing directory \"{output_directory:?}\""))?;
	}
	fs::create_dir_all(&output_directory)
		.with_context(|| format!("creating directory \"{output_directory:?}\""))?;

	scan(&input_directory, &output_directory)?;

	Ok(())
}

fn scan(input_directory: &Path, output_directory: &Path) -> Result<()> {
	let name_to_path =
		|name: &str| -> PathBuf { output_directory.join(name.to_lowercase().replace(" ", "_")) };

	let font_file = input_directory.join("fonts.json");
	if font_file.exists() {
		let data =
			fs::read(&font_file).with_context(|| format!("reading font file \"{font_file:?}\""))?;

		let font_configs = serde_json::from_slice::<Vec<FontConfig>>(&data)?;

		for font_config in font_configs {
			let mut renderer = FontRenderer::default();
			for source in font_config.sources {
				renderer.add_font_file(&input_directory.join(&source))?;
			}
			renderer.render_glyphs(&name_to_path(&font_config.name))?;
		}
	} else {
		for entry in fs::read_dir(&input_directory)? {
			let path = entry?.path();
			if path.is_file() {
				let extension = path.extension().unwrap_or_default().to_str().unwrap();
				if extension == "ttf" || extension == "otf" {
					let mut renderer = FontRenderer::default();
					renderer.add_font_file(&path)?;
					renderer
						.render_glyphs(&name_to_path(&path.file_stem().unwrap().to_str().unwrap()))?;
				}
			} else if path.is_dir() {
				scan(&path, output_directory)?;
			}
		}
	}

	Ok(())
}
