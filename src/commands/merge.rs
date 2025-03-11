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

#[cfg(test)]
mod tests {
	use super::*;

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
			input_files: vec![
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
