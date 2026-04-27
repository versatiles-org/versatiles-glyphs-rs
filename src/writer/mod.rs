//! Writers for storing glyph data in files or tar archives.

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
	finished: bool,
}

impl<'a> Writer<'a> {
	/// Creates a new `Writer` that writes to a tar archive.
	pub fn new_tar<W: std::io::Write + Send + Sync + 'static>(writer: &'a mut W) -> Self {
		Self {
			writer: Box::new(tar::TarWriter::new(writer)),
			finished: false,
		}
	}

	/// Creates a new `Writer` that writes to a directory on the filesystem.
	pub fn new_file(folder: std::path::PathBuf) -> Self {
		Self {
			writer: Box::new(file::FileWriter::new(folder)),
			finished: false,
		}
	}

	#[cfg(test)]
	/// Creates a new `Writer` that writes to an in-memory buffer.
	pub fn new_dummy() -> Self {
		Self {
			writer: Box::new(dummy::DummyWriter::default()),
			finished: false,
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
	///
	/// Idempotent: subsequent calls (including the implicit one in [`Drop`])
	/// are no-ops, so explicitly calling `finish()` will not produce duplicate
	/// trailers (e.g. extra zero-padding in a tar archive).
	pub fn finish(&mut self) -> Result<()> {
		if self.finished {
			return Ok(());
		}
		self.finished = true;
		self.writer.finish()
	}

	#[cfg(test)]
	/// Returns the inner buffer of the writer, if available.
	pub fn get_inner(&self) -> Option<&[String]> {
		self.writer.get_inner()
	}
}

impl Drop for Writer<'_> {
	/// Best-effort finalization. Drop cannot return an error, so a failure
	/// here is logged to stderr and the underlying I/O error is otherwise
	/// dropped. Callers that care about finalize errors should call
	/// [`Writer::finish`] explicitly.
	fn drop(&mut self) {
		if self.finished {
			return;
		}
		if let Err(e) = self.writer.finish() {
			eprintln!("warning: writer finalize failed during drop: {e:#}");
		}
		self.finished = true;
	}
}
