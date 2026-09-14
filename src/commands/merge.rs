use super::render_args::RenderArgs;
use crate::font::{FontManager, FontWrapper};
use anyhow::Result;
use std::{
	io::Write,
	path::{self, PathBuf},
};

/// Subcommand arguments for merging font files.
#[derive(clap::Args, Debug)]
#[command(arg_required_else_help = true, disable_version_flag = true)]
/// Merges one or more font files into a single font and writes its glyphs.
///
/// Sometimes fonts have to be split into multiple files since all characters for arabic, chinese, etc. do not fit in a single file.
/// This command merges all files into one font, regardless of their names, and writes its glyph ranges
/// (`{start}-{end}.pbf`) directly into the output directory or tar, without a font directory, `index.json` or `font_families.json`.
/// If several files contain the same character, the file listed first wins.
///
/// # Examples
///
/// ```bash
/// versatiles_glyphs merge -o output font.ttf
/// versatiles_glyphs merge -o output font.ttf font_arabic.ttf font_chinese.ttf
/// ```
pub struct Subcommand {
	/// One or more font files to merge and convert. Earlier files take precedence.
	#[arg(num_args=1..)]
	input_files: Vec<PathBuf>,

	#[command(flatten)]
	render: RenderArgs,
}

/// Executes the merge subcommand logic.
///
/// Loads all input files into a single [`FontWrapper`] and writes its glyph
/// blocks into the root of a directory or stdout tar.
pub fn run(args: &Subcommand, stdout: &mut (impl Write + Send + Sync + 'static)) -> Result<()> {
	// Canonicalize all input paths, keeping their order.
	let input_paths: Vec<PathBuf> = args
		.input_files
		.iter()
		.map(|p| Ok(path::absolute(p)?.canonicalize()?))
		.collect::<Result<Vec<_>>>()?;
	let font = FontWrapper::try_from(&input_paths[..])?;

	let mut writer = args.render.get_writer(stdout)?;
	let font_manager = FontManager::new(args.render.parallel());
	font_manager.render_font(&font, &mut writer, &args.render.get_renderer()?)?;
	writer.finish()?;

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::protobuf::{PbfGlyph, PbfGlyphs};
	use prost::Message;
	use std::fs;

	fn testdata(name: &str) -> PathBuf {
		PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.join("testdata")
			.join(name)
	}

	fn get_tar_entries(data: &[u8]) -> Vec<String> {
		let mut tar = tar::Archive::new(data);
		let mut entries = tar
			.entries()
			.unwrap()
			.filter_map(|e| {
				let e = e.unwrap();
				if e.size() == 2 {
					return None;
				}
				Some(format!("{:?}: {}", e.path().unwrap(), e.size()))
			})
			.collect::<Vec<_>>();
		entries.sort_unstable();
		entries
	}

	#[test]
	fn test_run_with_file_output() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let out = temp.path().join("glyphs");
		let args = Subcommand {
			input_files: vec![testdata("Fira Sans - Regular.ttf")],
			render: RenderArgs {
				output_directory: Some(out.to_str().unwrap().to_string()),
				dummy: true,
				..Default::default()
			},
		};

		run(&args, &mut Vec::<u8>::new())?;

		// Exactly one `.pbf` per BMP range directly in the output directory, nothing else.
		let mut names = fs::read_dir(&out)?
			.map(|e| e.unwrap().file_name().into_string().unwrap())
			.collect::<Vec<_>>();
		names.sort_unstable();
		assert_eq!(names.len(), 256);
		assert!(names.iter().all(|n| n.ends_with(".pbf")));
		assert!(out.join("0-255.pbf").is_file());
		assert!(out.join("65280-65535.pbf").is_file());
		Ok(())
	}

	#[test]
	fn test_run_with_tar_to_stdout() -> Result<()> {
		let args = Subcommand {
			input_files: vec![testdata("Fira Sans - Regular.ttf")],
			render: RenderArgs {
				tar: true,
				dummy: true,
				..Default::default()
			},
		};

		let mut stdout = Vec::<u8>::new();
		run(&args, &mut stdout)?;

		assert_eq!(
			get_tar_entries(&stdout),
			[
				"\"0-255.pbf\": 79996",
				"\"1024-1279.pbf\": 118007",
				"\"11264-11519.pbf\": 3547",
				"\"1280-1535.pbf\": 26266",
				"\"256-511.pbf\": 130722",
				"\"3584-3839.pbf\": 562",
				"\"42752-43007.pbf\": 5729",
				"\"43776-44031.pbf\": 455",
				"\"512-767.pbf\": 92606",
				"\"64256-64511.pbf\": 1000",
				"\"65024-65279.pbf\": 18",
				"\"7424-7679.pbf\": 7230",
				"\"768-1023.pbf\": 63731",
				"\"7680-7935.pbf\": 87048",
				"\"7936-8191.pbf\": 124490",
				"\"8192-8447.pbf\": 20271",
				"\"8448-8703.pbf\": 17365",
				"\"8704-8959.pbf\": 6481",
				"\"8960-9215.pbf\": 4345",
				"\"9472-9727.pbf\": 823"
			]
		);

		// 256 ranges and nothing else: no directory entries, no JSON files.
		let count = tar::Archive::new(&stdout[..]).entries()?.count();
		assert_eq!(count, 256);
		Ok(())
	}

	#[test]
	fn test_run_merges_different_families_into_one_font() -> Result<()> {
		let temp = tempfile::tempdir()?;
		let render = |name: &str, input_files: Vec<PathBuf>| -> Result<PathBuf> {
			let out = temp.path().join(name);
			let args = Subcommand {
				input_files,
				render: RenderArgs {
					output_directory: Some(out.to_str().unwrap().to_string()),
					dummy: true,
					..Default::default()
				},
			};
			run(&args, &mut Vec::<u8>::new())?;
			Ok(out)
		};

		// Different families (Fira Sans, Noto Sans Arabic) still end up in a single font.
		let fira = testdata("Fira Sans - Regular.ttf");
		let arabic = testdata("Noto Sans/Noto Sans Arabic - Regular.ttf");
		let merged = render("merged", vec![fira.clone(), arabic.clone()])?;
		let fira_only = render("fira", vec![fira])?;
		let arabic_only = render("arabic", vec![arabic])?;

		assert_eq!(fs::read_dir(&merged)?.count(), 256);
		// Arabic glyphs come from the Arabic font …
		assert_eq!(
			fs::read(merged.join("1536-1791.pbf"))?,
			fs::read(arabic_only.join("1536-1791.pbf"))?
		);
		// … and where both fonts have a glyph, it comes from Fira Sans, the first file.
		let decode = |path: PathBuf| -> Result<Vec<PbfGlyph>> {
			Ok(PbfGlyphs::decode(fs::read(path)?.as_slice())?.into_glyphs())
		};
		let merged_latin = decode(merged.join("0-255.pbf"))?;
		let fira_latin = decode(fira_only.join("0-255.pbf"))?;
		let arabic_latin = decode(arabic_only.join("0-255.pbf"))?;
		assert!(merged_latin.len() > fira_latin.len());
		for glyph in &fira_latin {
			assert!(
				merged_latin.contains(glyph),
				"U+{:04X} not from Fira",
				glyph.id
			);
		}
		// The glyphs Fira lacks are filled in from the Arabic font.
		for glyph in &merged_latin {
			assert!(fira_latin.contains(glyph) || arabic_latin.contains(glyph));
		}
		Ok(())
	}
}
