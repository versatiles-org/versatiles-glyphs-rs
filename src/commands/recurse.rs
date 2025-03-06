use anyhow::{Context, Result};
use serde::Deserialize;
use std::{
	fs,
	path::{self, Path},
};
use versatiles_glyphs::{
	font::FontManager,
	renderer::RendererPrecise,
	utils::prepare_output_directory,
	writer::{FileWriter, TarWriter, Writer},
};

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

	let input_directory = path::absolute(&arguments.input_directory)?.canonicalize()?;
	eprintln!("Scanning directory: {:?}", input_directory);
	scan(&input_directory, &mut font_manager)?;

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

fn scan(input_directory: &Path, font_manager: &mut FontManager) -> Result<()> {
	let font_file = input_directory.join("fonts.json");
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
					.map(|source| input_directory.join(source))
					.collect::<Vec<_>>(),
			)?;
		}
	} else {
		for entry in fs::read_dir(input_directory)? {
			let path = entry?.path();
			if path.is_file() {
				let extension = path.extension().unwrap_or_default().to_str().unwrap();
				if extension == "ttf" || extension == "otf" {
					font_manager.add_paths(&[path])?;
				}
			} else if path.is_dir() {
				scan(&path, font_manager)?;
			}
		}
	}

	Ok(())
}
