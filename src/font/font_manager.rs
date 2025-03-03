use crate::{
	font::{character_block::CharacterBlock, FontRenderer},
	utils::progress_bar::get_progress_bar,
	writer::Writer,
};
use anyhow::{bail, Result};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex_lite::Regex;
use std::{collections::HashMap, path::PathBuf, sync::Mutex};

pub struct FontManager<'a> {
	renderers: HashMap<String, FontRenderer<'a>>,
}

impl<'a> FontManager<'a> {
	pub fn new() -> Result<FontManager<'a>> {
		Ok(FontManager {
			renderers: HashMap::new(),
		})
	}

	pub fn add_font(&mut self, name: &str, sources: Vec<PathBuf>) -> Result<()> {
		let renderer = FontRenderer::from_paths(sources).unwrap();
		let id = name_to_id(name);
		if self.renderers.contains_key(&id) {
			bail!("Font with id \"{id}\" already exists");
		}
		self.renderers.insert(id, renderer);
		Ok(())
	}

	pub fn render_glyphs(&'a self, mut writer: Box<dyn Writer + Send + Sync>) -> Result<()> {
		let mut todos: Vec<(String, CharacterBlock<'a>)> = vec![];

		for (name, renderer) in &self.renderers {
			writer.write_directory(&format!("{name}/"))?;

			let blocks = renderer.get_blocks();
			for block in blocks {
				todos.push((name.clone(), block));
			}
		}

		let sum = todos.iter().map(|todo| todo.1.len() as u64).sum();
		let progress = get_progress_bar(sum);

		let tar_mutex = Mutex::new(writer);

		todos.par_iter().for_each(|todo| {
			let buf = todo.1.render(todo.0.clone()).unwrap();
			tar_mutex
				.lock()
				.unwrap()
				.write_file(&format!("{}/{}", todo.0, todo.1.filename()), &buf)
				.unwrap();
			progress.inc(todo.1.len() as u64);
		});

		progress.finish();

		Ok(())
	}
}

fn name_to_id(name: &str) -> String {
	let mut name = name.to_lowercase();
	name = Regex::new(r"[-_\s]+")
		.unwrap()
		.replace_all(&name, " ")
		.to_string()
		.trim()
		.to_string();
	name = name.replace(" ", "_");
	name
}
