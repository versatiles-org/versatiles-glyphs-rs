use anyhow::{Context, Result};
use std::{fs, path::PathBuf};

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
