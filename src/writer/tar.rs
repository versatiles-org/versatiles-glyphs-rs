use super::WriterTrait;
use anyhow::{ensure, Result};
use std::{
	io::{BufWriter, Write},
	time::{SystemTime, UNIX_EPOCH},
};

/// 1 KiB of zeros, used for padding data and finalizing the archive.
const ZEROS_1K: [u8; 1024] = [0; 1024];

/// A basic tar (POSIX.1-1988) archive writer that implements the [`Writer`] trait.
///
/// # Overview
/// This writer sequentially appends files and directories to an in-progress
/// tar archive. It accepts a generic `Write` output, making it flexible for
/// writing to files, buffers, network streams, etc.
///
/// # Features
/// - Generates minimal 512-byte headers containing filenames, file sizes,
///   timestamps, and checksums.
/// - Pads file data to 512-byte boundaries.
/// - Finalizes the tar file with an additional 1024 bytes of zero padding
///   as required by the format.
///
/// # Limitations
/// - Does not handle extended attributes, large file sizes, or other modern
///   tar features beyond POSIX.1-1988.
/// - Directories must end with a slash (`"/"`).
pub struct TarWriter<W: Write> {
	/// A buffered writer that collects and writes tar data.
	writer: BufWriter<W>,
}

impl<W: Write> TarWriter<W> {
	/// Creates a new [`TarWriter`] wrapping the provided `writer`.
	pub fn new(writer: W) -> Self {
		Self {
			writer: BufWriter::new(writer),
		}
	}

	/// Builds and writes a 512-byte tar header for a file or directory.
	///
	/// # Parameters
	/// - `path`: The path (file name or directory name).
	/// - `size`: The size of the file in bytes.
	/// - `mode`: The file mode (e.g., Unix permissions).
	/// - `typeflag`: Indicates file (`b'0'`) or directory (`b'5'`).
	fn write_header(&mut self, path: &str, size: u64, mode: u64, typeflag: u8) -> Result<()> {
		let mut header = [0u8; 512];

		// Name (bytes 0..100)
		write_string(&mut header[0..100], path);

		// File mode (octal, bytes 100..108)
		write_octal(&mut header[100..108], mode);

		// Owner's numeric user ID (octal, bytes 108..116)
		write_octal(&mut header[108..116], 0);

		// Group's numeric user ID (octal, bytes 116..124)
		write_octal(&mut header[116..124], 0);

		// File size in bytes (octal, bytes 124..136)
		write_octal(&mut header[124..136], size);

		// Last modification time in numeric Unix time (octal, bytes 136..148)
		let mtime = SystemTime::now()
			.duration_since(UNIX_EPOCH)
			.unwrap_or_default()
			.as_secs();
		write_octal(&mut header[136..148], mtime);

		// Type flag (file= '0', directory= '5'), byte 156
		header[156] = typeflag;

		// UStar magic (bytes 257..263) and version (263..265)
		header[257..263].copy_from_slice(b"ustar\0");
		header[263..265].copy_from_slice(b"00");

		// Fill the checksum field (148..156) with spaces
		#[allow(clippy::needless_range_loop)]
		for i in 148..156 {
			header[i] = b' ';
		}

		// Compute the header checksum
		let csum: u32 = header.iter().map(|&b| b as u32).sum();
		write_octal(&mut header[148..156], csum as u64);

		self.writer.write_all(&header)?;
		Ok(())
	}
}

impl<W: Write + Send + Sync> WriterTrait for TarWriter<W> {
	/// Writes `filename` and its associated `bytes` data as a file entry in the tar archive.
	///
	/// After writing the file contents, it pads to the next 512-byte boundary.
	///
	/// # Errors
	///
	/// Returns an error if writing the header or file data fails.
	fn write_file(&mut self, filename: &str, bytes: &[u8]) -> Result<()> {
		let size = bytes.len() as u64;
		self.write_header(filename, size, 0o644, b'0')?;
		self.writer.write_all(bytes)?;

		// Pad file contents to a 512-byte boundary
		let remainder = size % 512;
		if remainder != 0 {
			self
				.writer
				.write_all(&ZEROS_1K[0..(512 - remainder as usize)])?;
		}
		Ok(())
	}

	/// Writes a directory entry in the tar archive. The `dirname` must end with a slash.
	///
	/// # Errors
	///
	/// Returns an error if the `dirname` does not end with a slash or
	/// if writing the header fails.
	fn write_directory(&mut self, dirname: &str) -> Result<()> {
		ensure!(dirname.ends_with('/'), "dirname must end with a slash");
		self.write_header(dirname, 0, 0o755, b'5')?;
		Ok(())
	}

	/// Finalizes the tar archive by writing an extra 1024 bytes of zeros.
	///
	/// # Errors
	///
	/// Returns an error if the padding write fails.
	fn finish(&mut self) -> Result<()> {
		self.writer.write_all(&ZEROS_1K)?;
		self.writer.flush()?;
		Ok(())
	}

	#[cfg(test)]
	fn get_inner(&self) -> Option<&[String]> {
		None
	}
}

/// Writes an octal representation of `val` into `buf`, ending with a space character.
/// The buffer is filled from the right, and any remaining space on the left is filled with `0`.
fn write_octal(buf: &mut [u8], mut val: u64) {
	let len = buf.len();
	let mut idx = len - 1; // one before the final space
	buf[idx] = b' ';
	while idx > 0 {
		idx -= 1;
		buf[idx] = b'0' + (val & 7) as u8;
		val >>= 3;
	}
}

/// Copies the bytes of `s` into `dest`, truncating if necessary.
fn write_string(dest: &mut [u8], s: &str) {
	let bytes = s.as_bytes();
	let len = bytes.len().min(dest.len());
	dest[..len].copy_from_slice(&bytes[..len]);
}

#[cfg(test)]
mod tests {
	use super::*;
	use tar::{Archive, Entry};

	#[test]
	fn test_write_file() -> Result<()> {
		let mut output = Vec::new();
		{
			let mut tar = TarWriter::new(&mut output);
			tar.write_file("testfile.txt", b"hello tar")?;
			tar.finish()?;
		}

		assert_eq!(output.len(), 2048);

		assert_eq!(bytes_until_null(&output[0..100]), "testfile.txt");
		assert_eq!(output[156], b'0');
		assert_eq!(&output[512..512 + 9], b"hello tar");

		// Check that the file data is zero-padded up to a multiple of 512.
		for &b in &output[512 + 9..512 + 512] {
			assert_eq!(b, 0);
		}
		Ok(())
	}

	#[test]
	fn test_write_directory() -> Result<()> {
		let mut output = Vec::new();
		{
			let mut tar = TarWriter::new(&mut output);
			tar.write_directory("testdir/")?;
			tar.finish()?;
		}
		assert_eq!(output.len(), 1536);
		assert_eq!(bytes_until_null(&output[0..100]), "testdir/");

		// Check typeflag at byte 156. We expect b'5' for a directory.
		assert_eq!(output[156], b'5');

		// Check that the trailing zero blocks (the final 1024 bytes) are all zeros.
		let zeros_after_header = &output[512..];
		for &b in zeros_after_header {
			assert_eq!(b, 0);
		}
		Ok(())
	}

	#[test]
	fn test_multiple_files_and_finish() -> Result<()> {
		let mut output = Vec::new();
		{
			let mut tar = TarWriter::new(&mut output);
			tar.write_file("file1.txt", b"foo")?;
			tar.write_file("file2.txt", b"barbaz")?;
			tar.finish()?;
		}
		assert_eq!(output.len(), 3072);

		// Check the names in each header
		assert_eq!(bytes_until_null(&output[0..100]), "file1.txt");
		assert_eq!(bytes_until_null(&output[1024..1124]), "file2.txt");

		assert_eq!(&output[512..515], b"foo");
		assert_eq!(&output[1536..1542], b"barbaz");
		Ok(())
	}

	#[test]
	fn test_real_decoder() -> Result<()> {
		let mut output = Vec::new();
		{
			let mut tar = TarWriter::new(&mut output);
			tar.write_file("file1.txt", b"content 1")?;
			tar.write_directory("folder/")?;
			tar.write_file("file2.txt", b"content 2")?;
			tar.write_file("folder/file3.txt", b"content 3")?;
			tar.finish()?;
		}

		let mut archive = Archive::new(&output[..]);
		let entries = archive.entries()?.map(|e| e.unwrap()).collect::<Vec<_>>();
		assert_eq!(entries.len(), 4);
		assert_eq!(
			decode_entry(&entries, 0, &output)?,
				"type: Regular; path: 'file1.txt'; header_position: 0; file_position: 512; size: 9; content: 'content 1'"
		);
		assert_eq!(
			decode_entry(&entries, 1, &output)?,
				"type: Directory; path: 'folder/'; header_position: 1024; file_position: 1536; size: 0; content: ''"
		);
		assert_eq!(
			decode_entry(&entries, 2, &output)?,
				"type: Regular; path: 'file2.txt'; header_position: 1536; file_position: 2048; size: 9; content: 'content 2'"
		);
		assert_eq!(
			decode_entry(&entries, 3, &output)?,
				"type: Regular; path: 'folder/file3.txt'; header_position: 2560; file_position: 3072; size: 9; content: 'content 3'"
		);

		Ok(())
	}

	fn bytes_until_null(buf: &[u8]) -> &str {
		if let Some(pos) = buf.iter().position(|&b| b == 0) {
			std::str::from_utf8(&buf[..pos]).unwrap_or("")
		} else {
			std::str::from_utf8(buf).unwrap_or("")
		}
	}

	fn decode_entry(entries: &[Entry<'_, &[u8]>], index: usize, data: &[u8]) -> Result<String> {
		let entry = &entries[index];
		let path = entry.path()?.to_str().unwrap().to_string();
		let file_position = entry.raw_file_position() as usize;
		let header_position = entry.raw_header_position() as usize;
		let size = entry.size() as usize;
		let entry_type = entry.header().entry_type();
		let content = String::from_utf8_lossy(&data[file_position..file_position + size]).to_string();

		Ok(
			format!("type: {entry_type:?}; path: '{path}'; header_position: {header_position}; file_position: {file_position}; size: {size}; content: '{content}'"),
		)
	}
}
