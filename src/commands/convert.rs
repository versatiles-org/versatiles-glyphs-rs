use anyhow::{Context, Result};
use std::{fs, path};
use versatiles_glyphs::font::FontManager;

#[derive(clap::Args, Debug)]
#[command(arg_required_else_help = true, disable_version_flag = true)]
pub struct Subcommand {
	#[arg()]
	input_files: Vec<String>,

	#[arg(long, short = 'o')]
	output_directory: Option<String>,
}

pub fn run(arguments: &Subcommand) -> Result<()> {
	let directory = arguments.output_directory.as_deref().unwrap_or("output");
	let directory = path::absolute(directory)
		.with_context(|| format!("resolving output directory \"{directory}\""))?;

	if !directory.exists() {
		fs::create_dir_all(&directory)?;
	}

	let mut font_manager = FontManager::default();

	for input_file in arguments.input_files.iter() {
		let path = path::Path::new(input_file)
			.canonicalize()
			.with_context(|| format!("resolving font filename \"{input_file}\""))?;
		let data = fs::read(path).with_context(|| format!("reading font file \"{input_file}\""))?;
		font_manager
			.add_font(data)
			.with_context(|| format!("parsing font \"{input_file}\""))?;
	}

	font_manager
		.render_glyphs(&directory)
		.context("rendering glyphs")?;

	Ok(())
}
