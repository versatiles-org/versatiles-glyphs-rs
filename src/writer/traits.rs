use anyhow::Result;

pub trait Writer {
	fn write_file(&mut self, filename: &str, bytes: &[u8]) -> Result<()>;
	fn write_directory(&mut self, dirname: &str) -> Result<()>;
	fn finish(&mut self) -> Result<()>;
}
