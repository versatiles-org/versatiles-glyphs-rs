use anyhow::{Context, Result};
use std::{fs, path};
use versatiles_glyphs::font::FontRenderer;

#[derive(clap::Args, Debug)]
#[command(arg_required_else_help = true, disable_version_flag = true)]
/// Merges one or more font files and converts them into a single directory of glyphs.
///
/// Example: `versatiles_glyphs merge -o output font.ttf`
///
/// Sometimes fonts have to be split into multiple files since all characters for arabic, chinese, etc. do not fit in a single file.
/// In this case you can merge these files into a single directory of glyphs like so:
/// `versatiles_glyphs merge -o output font.ttf font_arabic.ttf font_chinese.ttf`
/// This command merges all these font files and converts them into a single directory of glyphs.
pub struct Subcommand {
	/// one or more font files to merge and convert
	#[arg(num_args=1..)]
	input_files: Vec<String>,

	/// the output directory where the glyphs will be saved.
	#[arg(long, short = 'o', default_value = "output")]
	output_directory: String,
}

pub fn run(arguments: &Subcommand) -> Result<()> {
	let directory = &arguments.output_directory;
	let directory = path::absolute(directory)
		.with_context(|| format!("resolving output directory \"{directory}\""))?;

	if !directory.exists() {
		fs::create_dir_all(&directory)?;
	}

	let input_files: Vec<&str> = arguments.input_files.iter().map(|s| s.as_str()).collect();
	let font = FontRenderer::from_filenames(input_files)?;

	font.render_glyphs(&directory).context("rendering glyphs")?;

	Ok(())
}
