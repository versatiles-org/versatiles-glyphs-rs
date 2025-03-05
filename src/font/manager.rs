use crate::{
	font::{GlyphBlock, FontFileEntry, FontRenderer},
	utils::get_progress_bar,
	writer::Writer,
};
use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex_lite::Regex;
use std::{collections::HashMap, path::PathBuf, sync::Mutex};
use super::index_files::{build_font_families_json, build_index_json};

pub struct FontManager<'a> {
	pub renderers: HashMap<String, FontRenderer<'a>>,
}

impl<'a> FontManager<'a> {
	pub fn new() -> FontManager<'a> {
		FontManager {
			renderers: HashMap::new(),
		}
	}

	pub fn add_fonts(&mut self, sources: Vec<PathBuf>) -> Result<()> {
		for source in sources {
			let font = FontFileEntry::new(std::fs::read(&source)?)?;
			let id = name_to_id(&font.metadata.generate_name());

			if let std::collections::hash_map::Entry::Vacant(e) = self.renderers.entry(id.clone()) {
				let mut renderer = FontRenderer::default();
				renderer.add_font(font);
				e.insert(renderer);
			} else {
				self.renderers.get_mut(&id).unwrap().add_font(font);
			}
		}
		Ok(())
	}

	pub fn add_font_with_name(&mut self, name: &str, sources: Vec<PathBuf>) -> Result<()> {
		let id = name_to_id(name);

		self
			.renderers
			.entry(id)
			.and_modify(|renderer| renderer.add_font_paths(&sources).unwrap())
			.or_insert_with(|| FontRenderer::from_paths(&sources).unwrap());
		Ok(())
	}

	pub fn render_glyphs(&'a self, writer: &mut Box<dyn Writer + Send + Sync>) -> Result<()> {
		let mut todos: Vec<(String, GlyphBlock<'a>)> = vec![];

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

	pub fn write_index_json(&self, writer: &mut Box<dyn Writer + Send + Sync>) -> Result<()> {
		writer.write_file("index.json", &build_index_json(self.renderers.iter())?)
	}

	pub fn write_families_json(&self, writer: &mut Box<dyn Writer + Send + Sync>) -> Result<()> {
		writer.write_file(
			"font_families.json",
			&build_font_families_json(self.renderers.iter())?,
		)
	}
}

fn name_to_id(name: &str) -> String {
	let mut name = name.to_lowercase();
	name = Regex::new(r"[-_\s]+")
		.unwrap()
		.replace_all(&name, " ")
		.trim()
		.to_string();
	name = name.replace(" ", "_");
	name
}
