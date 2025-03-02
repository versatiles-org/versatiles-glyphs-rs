use anyhow::Result;
// use std::{fs, path};
// use versatiles_glyphs::font::FontManager;

#[derive(clap::Args, Debug)]
#[command(arg_required_else_help = true)]
/// Scans a directory for font files and convert them into multiple directories of glyphs.
///
/// If a directory contains a "fonts.json" file, it will be used to configure the conversion.
/// A "fonts.json" has the structure: 	{ name: string, sources: string[] }[] where:
/// 	- name: the name of the font.
/// 	- sources: the list of font files to merge, relative to the directory.
pub struct Subcommand {
	/// directory to scan for font files.
	#[arg(num_args=1..)]
	input_directory: String,

	/// the output directory where the glyph folders will be saved.
	#[arg(long, short = 'o', default_value = "output")]
	output_directory: String,
}

pub fn run(arguments: &Subcommand) -> Result<()> {
	/*
	  let directory = &arguments.output_directory;
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
			  .add_font_data(data)
			  .with_context(|| format!("parsing font \"{input_file}\""))?;
	  }

	  font_manager
		  .render_glyphs(&directory)
		  .context("rendering glyphs")?;
	*/
	Ok(())
}
