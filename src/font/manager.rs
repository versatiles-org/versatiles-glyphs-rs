use super::index_files::{build_font_families_json, build_index_json};
use crate::{
	font::{FontFileEntry, FontWrapper, GlyphBlock},
	utils::get_progress_bar,
	writer::Writer,
};
use anyhow::Result;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use regex_lite::Regex;
use std::{
	collections::{hash_map::Entry, HashMap},
	path::PathBuf,
	sync::Mutex,
};

#[derive(Default)]
pub struct FontManager<'a> {
	pub fonts: HashMap<String, FontWrapper<'a>>,
}

impl<'a> FontManager<'a> {
	pub fn add_paths(&mut self, paths: &[PathBuf]) -> Result<()> {
		for path in paths {
			let file = FontFileEntry::new(std::fs::read(path)?)?;
			let id = name_to_id(&file.metadata.generate_name());

			if let Entry::Vacant(e) = self.fonts.entry(id.clone()) {
				let mut renderer = FontWrapper::default();
				renderer.add_file(file);
				e.insert(renderer);
			} else {
				self.fonts.get_mut(&id).unwrap().add_file(file);
			}
		}
		Ok(())
	}

	pub fn add_font_with_name(&mut self, name: &str, sources: &[PathBuf]) -> Result<()> {
		let id = name_to_id(name);

		self
			.fonts
			.entry(id)
			.and_modify(|renderer| renderer.add_paths(sources).unwrap())
			.or_insert_with(|| FontWrapper::try_from(sources).unwrap());
		Ok(())
	}

	pub fn render_glyphs(&'a self, writer: &mut Box<dyn Writer>) -> Result<()> {
		let mut todos: Vec<(String, GlyphBlock<'a>)> = vec![];

		for (name, renderer) in &self.fonts {
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

	pub fn write_index_json(&self, writer: &mut Box<dyn Writer>) -> Result<()> {
		writer.write_file("index.json", &build_index_json(self.fonts.iter())?)
	}

	pub fn write_families_json(&self, writer: &mut Box<dyn Writer>) -> Result<()> {
		writer.write_file(
			"font_families.json",
			&build_font_families_json(self.fonts.iter())?,
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
