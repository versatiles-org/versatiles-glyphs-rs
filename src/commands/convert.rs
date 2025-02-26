use anyhow::{Context, Result};
use sdf_glyphs::{composite, load_font_metadata, range_glyphs};
use ttf_parser::Face;

#[derive(clap::Args, Debug)]
#[command(arg_required_else_help = true, disable_version_flag = true)]
pub struct Subcommand {
	#[arg()]
	input_file: String,

	#[arg()]
	output_directory: String,
}

pub fn run(arguments: &Subcommand) -> Result<()> {
	println!("Hello, world! This is a pure-Rust skeleton port of glyphs.cpp.");

	// read file into memory
	let font_data = std::fs::read(&arguments.input_file)?;

	// Suppose you have font bytes in memory:
	let face = Face::parse(&font_data, 0).context("Could not parse font data")?;

	// 1) LOAD
	let metadata = load_font_metadata(&face)?;
	println!(
		"Loaded face: {} / style: {:?}; codepoints={}",
		metadata.family_name,
		metadata.style_name,
		metadata.codepoints.len()
	);

	// 2) RANGE
	let start_code = 32 as char;
	let end_code = 128 as char;
	let pbf_data = range_glyphs(&face, start_code, end_code)?;
	println!("Generated PBF of length: {}", pbf_data.len());

	// 3) COMPOSITE multiple buffers
	let composite_data = composite(&[pbf_data])?;
	println!("Composite PBF of length: {}", composite_data.len());

	Ok(())
}
