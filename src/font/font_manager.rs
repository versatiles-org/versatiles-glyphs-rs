use crate::{
	font::{character_block::CharacterBlock, FontFamily, FontRenderer},
	utils::get_progress_bar,
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

	pub fn add_font(&mut self, sources: Vec<PathBuf>) -> Result<()> {
		let renderer = FontRenderer::from_paths(sources).unwrap();
		let id = name_to_id(renderer.get_metadata().family.as_str());
		if self.renderers.contains_key(&id) {
			bail!("Font with id \"{id}\" already exists");
		}
		self.renderers.insert(id, renderer);
		Ok(())
	}

	pub fn add_font_with_name(&mut self, name: &str, sources: Vec<PathBuf>) -> Result<()> {
		let renderer = FontRenderer::from_paths(sources).unwrap();
		let id = name_to_id(name);
		if self.renderers.contains_key(&id) {
			bail!("Font with id \"{id}\" already exists");
		}
		self.renderers.insert(id, renderer);
		Ok(())
	}

	pub fn render_glyphs(&'a self, writer: &mut Box<dyn Writer + Send + Sync>) -> Result<()> {
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

	fn get_index(&self) -> Vec<String> {
		self.renderers.keys().cloned().collect()
	}
	fn get_families(&self) -> Vec<FontFamily> {
		let mut family_map = HashMap::<String, FontFamily>::new();
		for (id, renderer) in self.renderers.iter() {
			let meta = renderer.get_metadata();
			family_map
				.entry(meta.family.to_string())
				.or_insert_with(|| FontFamily::new(meta.family.to_string()))
				.add_font(
					id.to_string(),
					meta.style.to_string(),
					meta.weight,
					meta.width.to_string(),
				);
		}
		family_map.into_values().collect()
	}

	pub fn write_index_json(&self, writer: &mut Box<dyn Writer + Send + Sync>) -> Result<()> {
		let json = &serde_json::to_vec_pretty(&self.get_index())?;
		writer.write_file("index.json", json)
	}

	pub fn write_families_json(&self, writer: &mut Box<dyn Writer + Send + Sync>) -> Result<()> {
		let json = &serde_json::to_vec_pretty(&self.get_families())?;
		writer.write_file("font_families.json", json)
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
