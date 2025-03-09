use super::WriterTrait;
use anyhow::Result;
use regex_lite::Regex;

/// A dummy writer that captures written files and directories in memory
/// for testing or debugging purposes.
///
/// Instead of creating actual files, this writer records basic metadata
/// (file name, JSON content, or byte length) in a vector of strings.
/// This can be helpful for verifying logic without performing I/O.
#[derive(Default)]
pub struct DummyWriter {
	/// Stores textual information about each write operation.
	/// For `.json` files, the content is stripped of whitespace for brevity.
	data: Vec<String>,
}

impl WriterTrait for DummyWriter {
	/// Simulates writing a file by recording either the file name
	/// and its JSON content (condensed), or the file name with
	/// the number of bytes.
	///
	/// # Errors
	///
	/// Returns an error if `.json` content cannot be parsed as UTF-8
	/// or if the regular expression fails.
	fn write_file(&mut self, file_name: &str, bytes: &[u8]) -> Result<()> {
		let entry: String = if file_name.ends_with(".json") {
			let content = String::from_utf8(bytes.to_vec())?;
			// Remove excess newlines/whitespace
			let content = Regex::new(r#"\n\s*"#)?
				.replace_all(&content, "")
				.to_string();
			format!("{file_name}: {content}")
		} else {
			format!("{file_name} ({})", bytes.len())
		};
		self.data.push(entry);
		Ok(())
	}

	/// Simulates creating a directory by adding its name to `data`.
	fn write_directory(&mut self, dir_name: &str) -> Result<()> {
		self.data.push(dir_name.to_string());
		Ok(())
	}

	/// Finalizes the writer, but this dummy implementation does nothing.
	fn finish(&mut self) -> Result<()> {
		Ok(())
	}

	#[cfg(test)]
	fn get_inner(&self) -> Option<&[String]> {
		Some(&self.data)
	}
}
