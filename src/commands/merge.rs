use crate::{font::FontManager, render::Renderer, utils::prepare_output_directory, writer::Writer};
use anyhow::Result;
use std::{
	io::Write,
	path::{self, PathBuf},
};

/// Subcommand arguments for merging font files.
#[derive(clap::Args, Debug)]
#[command(arg_required_else_help = true, disable_version_flag = true)]
/// Merges one or more font files into a single directory or tar archive of glyphs.
///
/// Sometimes fonts have to be split into multiple files since all characters for arabic, chinese, etc. do not fit in a single file.
/// In this case you can merge these files into a single directory of glyphs like so:
/// `versatiles_glyphs merge -o output font.ttf font_arabic.ttf font_chinese.ttf`
/// This command merges all these font files and converts them into a single directory of glyphs.
///
/// # Examples
///
/// ```bash
/// versatiles_glyphs merge -o output font.ttf
/// versatiles_glyphs merge -o output font.ttf font_arabic.ttf font_chinese.ttf
/// ```
pub struct Subcommand {
	/// One or more font files to merge and convert.
	#[arg(num_args=1..)]
	input_files: Vec<PathBuf>,

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

/// Executes the merge subcommand logic.
///
/// Collects fonts, initializes a [`FontManager`], and writes glyph data
/// either to a directory or stdout tar.
pub fn run(args: &Subcommand, stdout: &mut (impl Write + Send + Sync + 'static)) -> Result<()> {
	let mut font_manager = FontManager::new(!args.single_thread);

	// Canonicalize all input paths before adding to the FontManager.
	let input_paths: Vec<PathBuf> = args
		.input_files
		.iter()
		.map(|p| Ok(path::absolute(p)?.canonicalize()?))
		.collect::<Result<Vec<_>>>()?;
	font_manager.add_paths(&input_paths)?;

	let mut writer = if args.tar {
		eprintln!("Rendering glyphs as tar to stdout.");
		Writer::new_tar(stdout)
	} else {
		let out_dir = prepare_output_directory(args.output_directory.as_deref().unwrap_or("output"))?;
		eprintln!("Rendering glyphs to directory: {:?}", out_dir);
		Writer::new_file(path::absolute(out_dir)?)
	};

	let renderer = Renderer::new(args.dummy);

	// Render glyphs and optionally write index/family files.
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
