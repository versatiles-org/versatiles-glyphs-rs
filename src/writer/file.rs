use super::traits::Writer;
use anyhow::{Context, Result};
use std::{fs::create_dir_all, path::PathBuf};

/// A simple tar (POSIX.1-1988) archive writer
pub struct FileWriter {
	folder: PathBuf,
}

impl FileWriter {
	pub fn new(folder: PathBuf) -> Self {
		Self { folder }
	}
}

impl Writer for FileWriter {
	fn write_file(&mut self, file_name: &str, bytes: &[u8]) -> Result<()> {
		let file_path = self.folder.join(file_name);
		std::fs::write(file_path, bytes)?;
		Ok(())
	}

	fn write_directory(&mut self, dir_name: &str) -> Result<()> {
		let dir_path = self.folder.join(dir_name);
		create_dir_all(&dir_path).with_context(|| format!("creating directory \"{dir_path:?}\""))?;
		Ok(())
	}

	fn finish(&mut self) -> Result<()> {
		Ok(())
	}

	#[cfg(test)]
	fn get_inner(&self) -> Option<&[String]> {
		None
	}
}
