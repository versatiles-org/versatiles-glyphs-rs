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

pub struct Writer<'a> {
	writer: Box<dyn WriterTrait + 'a>,
}

impl<'a> Writer<'a> {
	pub fn new_tar<W: std::io::Write + Send + Sync + 'static>(writer: &'a mut W) -> Self {
		Self {
			writer: Box::new(tar::TarWriter::new(writer)),
		}
	}
	pub fn new_file(folder: std::path::PathBuf) -> Self {
		Self {
			writer: Box::new(file::FileWriter::new(folder)),
		}
	}

	#[cfg(test)]
	pub fn new_dummy() -> Self {
		Self {
			writer: Box::new(dummy::DummyWriter::default()),
		}
	}

	pub fn write_file(&mut self, filename: &str, bytes: &[u8]) -> Result<()> {
		self.writer.write_file(filename, bytes)
	}

	pub fn write_directory(&mut self, dirname: &str) -> Result<()> {
		self.writer.write_directory(dirname)
	}

	pub fn finish(&mut self) -> Result<()> {
		self.writer.finish()
	}

	#[cfg(test)]
	pub fn get_inner(&self) -> Option<&[String]> {
		self.writer.get_inner()
	}
}

impl Drop for Writer<'_> {
	fn drop(&mut self) {
		self.finish().unwrap();
	}
}
