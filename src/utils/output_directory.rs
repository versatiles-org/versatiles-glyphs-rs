use anyhow::{Context, Result};
use std::{fs, path::PathBuf};

/// Prepares a fresh output directory by removing any existing contents,
/// then creating a new directory at the specified path.
///
/// # Arguments
///
/// * `output_directory` - The string path of the directory to create.
///
/// # Returns
///
/// A [`PathBuf`] referencing the newly created directory.
///
/// # Errors
///
/// Returns an error if:
/// - The existing directory could not be removed.
/// - The new directory could not be created.
///
/// This operation is considered destructive, because any existing directory
/// (and its contents) at `output_directory` will be removed.
///
/// # Examples
///
/// ```
/// use versatiles_glyphs::utils::prepare_output_directory;
/// use std::path::PathBuf;
///
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// let output_path = prepare_output_directory("target/output")?;
/// assert!(output_path.exists());
/// # Ok(())
/// # }
/// ```
pub fn prepare_output_directory(output_directory: &str) -> Result<PathBuf> {
	let output_directory: PathBuf = PathBuf::from(output_directory);

	if output_directory.exists() {
		fs::remove_dir_all(&output_directory)
			.with_context(|| format!("removing directory \"{output_directory:?}\""))?;
	}
	fs::create_dir_all(&output_directory)
		.with_context(|| format!("creating directory \"{output_directory:?}\""))?;

	Ok(output_directory)
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::io::Write;
	use tempfile::tempdir;

	#[test]
	fn test_prepare_output_directory_when_not_exists() {
		let temp_dir = tempdir().unwrap();
		let output_dir = temp_dir.path().join("non_existent_dir");
		let output_dir_str = output_dir.to_str().unwrap();

		assert!(!output_dir.exists());

		let prepared_dir = prepare_output_directory(output_dir_str).unwrap();

		assert!(prepared_dir.exists());
		assert!(prepared_dir.is_dir());

		let entries: Vec<_> = fs::read_dir(&prepared_dir).unwrap().collect();
		assert!(entries.is_empty());
	}

	#[test]
	fn test_prepare_output_directory_when_exists() {
		let temp_dir = tempdir().unwrap();
		let output_dir = temp_dir.path().join("existing_dir");
		fs::create_dir_all(&output_dir).unwrap();

		let dummy_file_path = output_dir.join("dummy.txt");
		let mut file = fs::File::create(&dummy_file_path).unwrap();
		writeln!(file, "dummy content").unwrap();
		file.sync_all().unwrap();

		assert!(dummy_file_path.exists());

		let output_dir_str = output_dir.to_str().unwrap();
		let prepared_dir = prepare_output_directory(output_dir_str).unwrap();

		assert!(prepared_dir.exists());
		assert!(prepared_dir.is_dir(),);

		let entries: Vec<_> = fs::read_dir(&prepared_dir).unwrap().collect();
		assert!(entries.is_empty(),);
	}
}
