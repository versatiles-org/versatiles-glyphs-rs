use anyhow::{Context, Result};
use std::{
	fs::{create_dir_all, remove_dir_all},
	path::PathBuf,
};

pub fn prepare_output_directory(output_directory: &str) -> Result<PathBuf> {
	let output_directory: PathBuf = PathBuf::from(output_directory);

	if output_directory.exists() {
		remove_dir_all(&output_directory)
			.with_context(|| format!("removing directory \"{output_directory:?}\""))?;
	}
	create_dir_all(&output_directory)
		.with_context(|| format!("creating directory \"{output_directory:?}\""))?;

	Ok(output_directory)
}
