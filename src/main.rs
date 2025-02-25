mod libraries;

use anyhow::Result;
use libraries::{composite, load_font, range_glyphs};

// ---------------------------------------------------------
// Demo entry point
// ---------------------------------------------------------

fn main() -> Result<()> {
	println!("Hello, world! This is a pure-Rust skeleton port of glyphs.cpp.");

	// Suppose you have font bytes in memory:
	let fake_font_data: &[u8] = include_bytes!("../testdata/Fira Sans - Regular.ttf");

	// 1) LOAD
	let faces = load_font(fake_font_data)?;
	for face in faces {
		println!(
			"Loaded face: {} / style: {:?}; codepoints={}",
			face.family_name,
			face.style_name,
			face.points.len()
		);
	}

	// 2) RANGE
	let start_code = 32 as char;
	let end_code = 128 as char;
	let pbf_data = range_glyphs(fake_font_data, start_code, end_code)?;
	println!("Generated PBF of length: {}", pbf_data.len());

	// 3) COMPOSITE multiple buffers
	let composite_data = composite(&[pbf_data])?;
	println!("Composite PBF of length: {}", composite_data.len());

	Ok(())
}
