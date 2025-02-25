use anyhow::Result;
use flate2::bufread::GzDecoder;
use std::io::Read;

/// Attempt to detect if the input is gzip compressed (just checking magic bytes).
pub fn maybe_decompress(data: &[u8]) -> Result<Vec<u8>> {
	const GZIP_MAGIC: [u8; 2] = [0x1F, 0x8B];
	if data.len() >= 2 && data[0..2] == GZIP_MAGIC {
		let mut d = GzDecoder::new(data);
		let mut out = Vec::new();
		d.read_to_end(&mut out)?;
		Ok(out)
	} else {
		Ok(data.to_vec())
	}
}
