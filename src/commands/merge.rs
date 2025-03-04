use anyhow::Result;
use std::path::{self, PathBuf};
use versatiles_glyphs::{
	font::FontManager,
	utils::prepare_output_directory,
	writer::{FileWriter, TarWriter, Writer},
};

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

	/// the output directory where the glyphs folder will be saved.
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

pub fn run(arguments: &Subcommand) -> Result<()> {
	let mut font_manager = FontManager::new()?;

	let input_paths: Vec<PathBuf> = arguments
		.input_files
		.iter()
		.map(|input_file| -> Result<PathBuf> { Ok(path::absolute(input_file)?.canonicalize()?) })
		.collect::<Result<Vec<PathBuf>>>()?;
	font_manager.add_font(input_paths)?;

	let mut writer: Box<dyn Writer + Send + Sync> = if arguments.tar {
		eprintln!("Rendering glyphs as tar to stdout");
		Box::new(TarWriter::new(std::io::stdout()))
	} else {
		let output_directory =
			prepare_output_directory(arguments.output_directory.as_deref().unwrap_or("output"))?;

		eprintln!("Rendering glyphs to directory: {:?}", output_directory);
		Box::new(FileWriter::new(path::absolute(output_directory)?))
	};

	font_manager.render_glyphs(&mut writer)?;
	if !arguments.no_index {
		font_manager.write_index_json(&mut writer)?;
	}
	if !arguments.no_families {
		font_manager.write_families_json(&mut writer)?;
	}

	writer.finish()?;

	Ok(())
}
