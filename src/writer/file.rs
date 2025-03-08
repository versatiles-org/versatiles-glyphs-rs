use super::traits::Writer;
use anyhow::{Context, Result};
use std::{fs::create_dir_all, path::PathBuf};

/// Writes data directly to the filesystem.
///
/// This writer takes a user-specified folder and creates files or
/// subdirectories within it. It implements the [`Writer`] trait,
/// enabling it to be swapped for dummy or tar-based writers.
pub struct FileWriter {
	/// The root folder where files and subdirectories are written.
	folder: PathBuf,
}

impl FileWriter {
	/// Creates a new [`FileWriter`] that will use the given `folder`
	/// as its root.
	///
	/// # Examples
	///
	/// ```
	/// use versatiles_glyphs::writer::file::FileWriter;
	/// use std::path::PathBuf;
	///
	/// let writer = FileWriter::new(PathBuf::from("output"));
	/// ```
	pub fn new(folder: PathBuf) -> Self {
		Self { folder }
	}
}

impl Writer for FileWriter {
	/// Creates (or overwrites) a file with the given `file_name`
	/// under the writer's root folder, writing the provided `bytes`.
	///
	/// # Errors
	///
	/// Fails if the file cannot be created or written.
	fn write_file(&mut self, file_name: &str, bytes: &[u8]) -> Result<()> {
		let file_path = self.folder.join(file_name);
		std::fs::write(file_path, bytes)?;
		Ok(())
	}

	/// Creates a directory with the given name under the writer's root folder,
	/// including any intermediate directories if needed.
	///
	/// # Errors
	///
	/// Fails if the directory cannot be created.
	fn write_directory(&mut self, dir_name: &str) -> Result<()> {
		let dir_path = self.folder.join(dir_name);
		create_dir_all(&dir_path).with_context(|| format!("creating directory \"{dir_path:?}\""))?;
		Ok(())
	}

	/// Concludes writing. For a [`FileWriter`] this is a no-op.
	fn finish(&mut self) -> Result<()> {
		Ok(())
	}

	#[cfg(test)]
	fn get_inner(&self) -> Option<&[String]> {
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::fs;
	use tempfile::tempdir;

	#[test]
	fn test_write_file() -> Result<()> {
		let temp_dir = tempdir()?;
		let folder_path = temp_dir.path().to_path_buf();
		let mut writer = FileWriter::new(folder_path.clone());

		let file_name = "test.txt";
		let content = b"Hello, FileWriter!";
		writer.write_file(file_name, content)?;

		let written_file_path = folder_path.join(file_name);
		let written_content = fs::read(written_file_path)?;
		assert_eq!(written_content, content);
		Ok(())
	}

	#[test]
	fn test_write_directory() -> Result<()> {
		let temp_dir = tempdir()?;
		let folder_path = temp_dir.path().to_path_buf();
		let mut writer = FileWriter::new(folder_path.clone());

		let dir_name = "subdir";
		writer.write_directory(dir_name)?;

		let dir_path = folder_path.join(dir_name);
		assert!(dir_path.exists());
		assert!(dir_path.is_dir());
		Ok(())
	}

	#[test]
	fn test_finish() -> Result<()> {
		let temp_dir = tempdir()?;
		let folder_path = temp_dir.path().to_path_buf();
		let mut writer = FileWriter::new(folder_path);

		writer.finish()?;
		Ok(())
	}
}
