use anyhow::{Context, Result};
use sdf_glyphs::{load_font_metadata, range_glyphs};
use ttf_parser::Face;

#[derive(clap::Args, Debug)]
#[command(arg_required_else_help = true, disable_version_flag = true)]
pub struct Subcommand {
	#[arg()]
	input_file: String,

	#[arg()]
	output_directory: String,
}

const GROUP_SIZE: u32 = 256;

pub fn run(arguments: &Subcommand) -> Result<()> {
	println!("Hello, world! This is a pure-Rust skeleton port of glyphs.cpp.");

	let directory = std::path::Path::new(&arguments.output_directory);
	if !directory.exists() {
		std::fs::create_dir_all(directory)?;
	}

	// read file into memory
	let font_data = std::fs::read(&arguments.input_file)?;

	// Suppose you have font bytes in memory:
	let face = Face::parse(&font_data, 0).context("Could not parse font data")?;

	// 1) LOAD
	let metadata = load_font_metadata(&face)?;
	println!(
		"Loaded face: \"{}\", style: \"{}\", codepoints: {}",
		metadata.family_name,
		metadata.style_name,
		metadata.codepoints.len()
	);

	let mut groups = std::collections::HashMap::<u32, u32>::new();
	metadata.codepoints.iter().for_each(|cp| {
		let group = cp / GROUP_SIZE;
		if groups.contains_key(&group) {
			*groups.get_mut(&group).unwrap() += 1;
		} else {
			groups.insert(group, 1);
		}
	});
	let mut groups = groups.into_iter().collect::<Vec<(u32, u32)>>();
	groups.sort();

	let progress = indicatif::ProgressBar::new(metadata.codepoints.len() as u64);
	progress.set_position(0);
	for (group, count) in groups.iter() {
		let start_index = group * GROUP_SIZE;
		let end_index = (group + 1) * GROUP_SIZE - 1;
		let pbf_data = range_glyphs(&face, start_index, end_index)?;

		let filename = format!("{}-{}.pbf", start_index, end_index);
		std::fs::write(directory.join(filename), pbf_data)?;
		progress.inc(*count as u64);
	}
	progress.finish();

	Ok(())
}
