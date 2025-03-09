//! Provides implementations of the [`Writer`](writer::Writer) trait for various output targets.

#[cfg(test)]
mod dummy;
mod file;
mod tar;

use anyhow::Result;

trait WriterTrait
where
	Self: Send + Sync,
{
	fn write_file(&mut self, filename: &str, bytes: &[u8]) -> Result<()>;
	fn write_directory(&mut self, dirname: &str) -> Result<()>;
	fn finish(&mut self) -> Result<()>;
	#[cfg(test)]
	fn get_inner(&self) -> Option<&[String]>;
}

/// A struct for writing files and directories to various output targets.
pub struct Writer<'a> {
	writer: Box<dyn WriterTrait + 'a>,
}

impl<'a> Writer<'a> {
	/// Creates a new `Writer` that writes to a tar archive.
	pub fn new_tar<W: std::io::Write + Send + Sync + 'static>(writer: &'a mut W) -> Self {
		Self {
			writer: Box::new(tar::TarWriter::new(writer)),
		}
	}

	/// Creates a new `Writer` that writes to a directory on the filesystem.
	pub fn new_file(folder: std::path::PathBuf) -> Self {
		Self {
			writer: Box::new(file::FileWriter::new(folder)),
		}
	}

	#[cfg(test)]
	/// Creates a new `Writer` that writes to an in-memory buffer.
	pub fn new_dummy() -> Self {
		Self {
			writer: Box::new(dummy::DummyWriter::default()),
		}
	}

	/// Writes the given bytes to a file with the given filename.
	pub fn write_file(&mut self, filename: &str, bytes: &[u8]) -> Result<()> {
		self.writer.write_file(filename, bytes)
	}

	/// Writes an empty directory with the given name.
	pub fn write_directory(&mut self, dirname: &str) -> Result<()> {
		self.writer.write_directory(dirname)
	}

	/// Finishes writing to the output target.
	pub fn finish(&mut self) -> Result<()> {
		self.writer.finish()
	}

	#[cfg(test)]
	/// Returns the inner buffer of the writer, if available.
	pub fn get_inner(&self) -> Option<&[String]> {
		self.writer.get_inner()
	}
}

impl Drop for Writer<'_> {
	fn drop(&mut self) {
		self.finish().unwrap();
	}
}
