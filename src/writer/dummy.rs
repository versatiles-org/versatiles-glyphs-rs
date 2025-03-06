use super::traits::Writer;
use anyhow::Result;
use regex_lite::Regex;

#[derive(Default)]
pub struct DummyWriter {
	data: Vec<String>,
}

impl Writer for DummyWriter {
	fn write_file(&mut self, file_name: &str, bytes: &[u8]) -> Result<()> {
		let entry: String = if file_name.ends_with(".json") {
			let content = String::from_utf8(bytes.to_vec())?;
			let content = Regex::new(r#"\n\s*"#)?
				.replace_all(&content, "")
				.to_string();
			format!("{file_name}: {}", content)
		} else {
			format!("{file_name} ({})", bytes.len())
		};
		self.data.push(entry);
		Ok(())
	}

	fn write_directory(&mut self, dir_name: &str) -> Result<()> {
		self.data.push(dir_name.to_string());
		Ok(())
	}

	fn finish(&mut self) -> Result<()> {
		Ok(())
	}

	#[cfg(test)]
	fn get_inner(&self) -> Option<&[String]> {
		Some(&self.data)
	}
}
