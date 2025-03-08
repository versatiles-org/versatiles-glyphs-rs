use anyhow::Result;

/// A trait for writing files and directories, suitable for various backends
/// (e.g., to disk, to a tar archive, or purely in-memory).
///
/// # Required Methods
/// - [`write_file`](Writer::write_file): Writes the provided `bytes` under `filename`.
/// - [`write_directory`](Writer::write_directory): Creates or declares a directory.
/// - [`finish`](Writer::finish): Performs any necessary finalization, such as flushing buffers.
pub trait Writer
where
	Self: Send + Sync,
{
	/// Writes the provided `bytes` to `filename`.
	///
	/// # Errors
	///
	/// Returns an error if the file cannot be written.
	fn write_file(&mut self, filename: &str, bytes: &[u8]) -> Result<()>;

	/// Declares or creates `dirname` as a directory.
	///
	/// # Errors
	///
	/// Returns an error if the directory cannot be created.
	fn write_directory(&mut self, dirname: &str) -> Result<()>;

	/// Completes any outstanding operations, flushing buffers or
	/// closing open resources.
	fn finish(&mut self) -> Result<()>;

	/// Exposes underlying data for testing purposes, if available.
	#[cfg(test)]
	fn get_inner(&self) -> Option<&[String]>;
}
