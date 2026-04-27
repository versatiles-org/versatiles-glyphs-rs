use crate::protobuf::PbfGlyphs;
use anyhow::{bail, Context, Result};
use clap::ValueEnum;
use prost::Message;
use std::{fs, io::Write, path::PathBuf};

#[derive(Clone, Debug, ValueEnum)]
enum Format {
	Csv,
	Tsv,
}

/// Subcommand arguments for recursively scanning font files.
#[derive(clap::Args, Debug)]
#[command(arg_required_else_help = true)]
/// Recursively scans directories for `.ttf` or `.otf` files and converts them.
///
/// If a directory contains a "fonts.json" file, it will be used to configure the conversion.
/// A "fonts.json" has the structure: { name: string, sources: string[] }[] where:
///   - name: the name of the font, like "Noto Sans Regular".
///   - sources: the list of font files to merge, relative to the directory.
///
/// # Examples
///
/// ```bash
/// versatiles_glyphs recurse -o glyphs my_font_directory
/// versatiles_glyphs recurse -t another_directory
/// ```
pub struct Subcommand {
	/// Directories to scan for font files.
	#[arg()]
	glyph_directory: PathBuf,

	#[arg(short, long, default_value = "csv")]
	format: Format,
}

pub fn run(args: &Subcommand, stdout: &mut (impl Write + Send + Sync + 'static)) -> Result<()> {
	let glyph_directory = &args.glyph_directory;

	if !glyph_directory.exists() {
		bail!("Directory does not exist: {:?}", glyph_directory);
	}

	let mut write = |out: [String; 7]| {
		writeln!(
			stdout,
			"{}",
			match args.format {
				Format::Csv => out.join(","),
				Format::Tsv => out.join("\t"),
			}
		)
	};

	write([
		String::from("codepoint"),
		String::from("width"),
		String::from("height"),
		String::from("left"),
		String::from("top"),
		String::from("advance"),
		String::from("bitmap_size"),
	])?;

	for i in 0..256 {
		let start = i * 256;
		let end = start + 255;
		let filename = glyph_directory.join(format!("{start}-{end}.pbf"));
		let buf = match fs::read(&filename) {
			Ok(b) => b,
			Err(e) if e.kind() == std::io::ErrorKind::NotFound => continue,
			Err(e) => return Err(e).with_context(|| format!("Failed to read {filename:?}")),
		};
		let mut glyphs = PbfGlyphs::decode(buf.as_slice())
			.with_context(|| format!("Failed to decode {filename:?}"))?
			.into_glyphs();

		glyphs.sort_unstable();

		for glyph in glyphs {
			write([
				glyph.id.to_string(),
				glyph.width.to_string(),
				glyph.height.to_string(),
				glyph.left.to_string(),
				glyph.top.to_string(),
				glyph.advance.to_string(),
				glyph
					.bitmap
					.as_ref()
					.map_or("0".to_string(), |b| b.len().to_string()),
			])?;
		}
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{font::FontManager, render::Renderer, writer::Writer};
	use tempfile::tempdir;

	/// End-to-end smoke test for `debug::run`. Renders Fira Sans into a tempdir
	/// and parses the resulting `.pbf` files back via the `debug` subcommand.
	///
	/// Also exercises the sparse-range fix in `run`: the rendered output covers
	/// only ~20 of the 256 possible 256-codepoint ranges, so `run` must skip
	/// missing range files instead of erroring on the first gap.
	#[test]
	fn test_debug_run_csv_on_rendered_font() -> Result<()> {
		let temp = tempdir()?;

		// 1) Render Fira Sans to <temp>/fira_sans_regular/*.pbf using the dummy
		//    renderer (fast; produces valid PBF structure with empty bitmaps).
		let mut manager = FontManager::new(false);
		manager.add_path(
			&PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("testdata/Fira Sans - Regular.ttf"),
		)?;
		let mut writer = Writer::new_file(temp.path().to_path_buf());
		manager.render_glyphs(&mut writer, &Renderer::new_dummy())?;
		writer.finish()?;

		let glyph_dir = temp.path().join("fira_sans_regular");
		assert!(
			glyph_dir.is_dir(),
			"render did not create the expected font subdir"
		);

		// 2) Run debug::run against the sparse output.
		let args = Subcommand {
			glyph_directory: glyph_dir,
			format: Format::Csv,
		};
		let mut stdout: Vec<u8> = Vec::new();
		run(&args, &mut stdout)?;

		// 3) Verify shape: header + many data rows.
		let output = String::from_utf8(stdout)?;
		let mut lines = output.lines();
		assert_eq!(
			lines.next(),
			Some("codepoint,width,height,left,top,advance,bitmap_size")
		);
		let row_count = lines.count();
		assert!(
			row_count > 1000,
			"expected >1000 glyph rows from Fira Sans, got {row_count}"
		);
		Ok(())
	}

	#[test]
	fn test_debug_run_missing_directory_errors() {
		let args = Subcommand {
			glyph_directory: PathBuf::from("/nonexistent/path/that/should/not/exist"),
			format: Format::Csv,
		};
		let mut stdout: Vec<u8> = Vec::new();
		let err = run(&args, &mut stdout).unwrap_err();
		assert!(err.to_string().contains("Directory does not exist"));
	}
}
