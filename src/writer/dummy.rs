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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_write_directory_records_name() {
		let mut w = DummyWriter::default();
		w.write_directory("subdir/").unwrap();
		assert_eq!(w.get_inner().unwrap(), &["subdir/".to_string()]);
	}

	#[test]
	fn test_write_non_json_records_byte_length() {
		let mut w = DummyWriter::default();
		w.write_file("data.pbf", &[0u8; 42]).unwrap();
		assert_eq!(w.get_inner().unwrap(), &["data.pbf (42)".to_string()]);
	}

	#[test]
	fn test_write_json_records_condensed_content() {
		let mut w = DummyWriter::default();
		w.write_file("config.json", b"{\n  \"a\": 1,\n  \"b\": 2\n}")
			.unwrap();
		assert_eq!(
			w.get_inner().unwrap(),
			&[r#"config.json: {"a": 1,"b": 2}"#.to_string()]
		);
	}

	#[test]
	fn test_write_json_invalid_utf8_errors() {
		let mut w = DummyWriter::default();
		let err = w.write_file("bad.json", &[0xff, 0xfe]).unwrap_err();
		// `from_utf8` produces a `FromUtf8Error`; anyhow wraps it.
		assert!(err.is::<std::string::FromUtf8Error>());
	}

	#[test]
	fn test_finish_is_noop() {
		let mut w = DummyWriter::default();
		w.finish().unwrap();
	}
}
