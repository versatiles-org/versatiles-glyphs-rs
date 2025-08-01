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
		let buf = fs::read(&filename).with_context(|| format!("Failed to read {filename:?}"))?;
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

	#[test]
	fn test_run_with_csv() -> Result<()> {
		Ok(())
	}
}
