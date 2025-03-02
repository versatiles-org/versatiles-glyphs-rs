use super::{character_block::CharacterBlock, FontRenderer};
use anyhow::{Context, Result};
use indicatif::ProgressStyle;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
	fs::create_dir_all,
	path::{Path, PathBuf},
};

pub struct FontManager<'a> {
	renderers: Vec<(String, FontRenderer<'a>)>,
}

impl<'a> FontManager<'a> {
	pub fn new() -> Result<FontManager<'a>> {
		Ok(FontManager {
			renderers: Vec::new(),
		})
	}

	pub fn add_font(&mut self, name: &str, sources: Vec<PathBuf>) {
		let renderer = FontRenderer::from_paths(sources).unwrap();
		self.renderers.push((name.to_string(), renderer));
	}

	pub fn render_glyphs(&'a self, directory: &Path) -> Result<()> {
		create_dir_all(&directory)
			.with_context(|| format!("creating directory \"{directory:?}\""))?;

		let mut todos: Vec<(PathBuf, CharacterBlock<'a>)> = vec![];

		for (name, renderer) in &self.renderers {
			let path = directory.join(name.to_lowercase().replace(" ", "_"));
			create_dir_all(&path)?;

			let blocks = renderer.get_chunks();
			for block in blocks {
				todos.push((path.clone(), block));
			}
		}

		let sum = todos.iter().map(|todo| todo.1.len() as u64).sum();
		let progress = indicatif::ProgressBar::new(sum)
			.with_position(0)
			.with_style(ProgressStyle::with_template(
				"{wide_bar} {pos:>8}/{len:8} {eta_precise:8}",
			)?);

		todos.par_iter().for_each(|todo| {
			todo.1.render_to_file(&todo.0).unwrap();
			progress.inc(todo.1.len() as u64);
		});

		progress.finish();

		Ok(())
	}
}
