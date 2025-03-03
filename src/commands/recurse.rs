use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
	fs::{self, create_dir_all},
	path::{self, Path},
};
use versatiles_glyphs::{font::FontManager, utils::TarWriter};

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
	#[arg(long, short = 'o', conflicts_with = "tar")]
	output_directory: Option<String>,

	/// the output directory where the glyph folders will be saved.
	#[arg(long, short = 't', conflicts_with = "output_directory")]
	tar: bool,
}

#[derive(Debug, Deserialize)]
struct FontConfig {
	name: String,
	sources: Vec<String>,
}

pub fn run(arguments: &Subcommand) -> Result<()> {
	let mut font_manager = FontManager::new()?;

	let input_directory = path::absolute(&arguments.input_directory)?.canonicalize()?;
	eprintln!("Scanning directory: {:?}", input_directory);
	scan(&input_directory, &mut font_manager)?;

	if arguments.tar {
		let mut tar = TarWriter::new(std::io::stdout());
		eprintln!("Rendering glyphs as tar to stdout");
		font_manager.render_glyphs_to_tar(&mut tar)?;
		tar.finish()?;
		return Ok(());
	}

	let output_directory = arguments
		.output_directory
		.as_ref()
		.map(|f| f.clone())
		.unwrap_or_else(|| String::from("output"));

	let mut output_directory = path::absolute(output_directory)?;
	if output_directory.exists() {
		fs::remove_dir_all(&output_directory)
			.with_context(|| format!("removing directory \"{output_directory:?}\""))?;
	}
	create_dir_all(&output_directory)
		.with_context(|| format!("creating directory \"{output_directory:?}\""))?;
	output_directory = output_directory.canonicalize()?;

	eprintln!("Rendering glyphs to directory: {:?}", output_directory);
	font_manager.render_glyphs_to_dir(&output_directory)?;

	Ok(())
}

fn scan(input_directory: &Path, font_manager: &mut FontManager) -> Result<()> {
	let font_file = input_directory.join("fonts.json");
	if font_file.exists() {
		let data =
			fs::read(&font_file).with_context(|| format!("reading font file \"{font_file:?}\""))?;

		let font_configs = serde_json::from_slice::<Vec<FontConfig>>(&data)?;

		for font_config in font_configs {
			font_manager.add_font(
				&font_config.name,
				font_config
					.sources
					.iter()
					.map(|source| input_directory.join(source))
					.collect(),
			)?;
		}
	} else {
		for entry in fs::read_dir(input_directory)? {
			let path = entry?.path();
			if path.is_file() {
				let extension = path.extension().unwrap_or_default().to_str().unwrap();
				if extension == "ttf" || extension == "otf" {
					let name = path.file_stem().unwrap().to_str().unwrap().to_string();
					font_manager.add_font(&name, vec![path])?;
				}
			} else if path.is_dir() {
				scan(&path, font_manager)?;
			}
		}
	}

	Ok(())
}
