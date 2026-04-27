/// Converts a grayscale bitmap into rows of two-digit numeric strings,
/// each representing the pixel's grayscale value scaled to `0..=99`.
///
/// Each pixel `x` is mapped to `(x * 100) / 256`, capped at 99, then
/// formatted as a zero-padded two-digit string. For instance, a value
/// of `128` becomes `"50"` and `255` becomes `"99"`.
///
/// # Arguments
/// * `bitmap` - A slice of 8-bit grayscale pixel data.
/// * `width` - The number of pixels per row.
///
/// # Returns
/// A vector of strings, where each string represents one row of the image
/// (with each pixel replaced by two digits and separated by spaces).
pub fn bitmap_as_digit_art(bitmap: &[u8], width: usize) -> Vec<String> {
	bitmap
		.chunks(width)
		.map(|row| {
			row.iter()
				.map(|&x| {
					let v = ((x as u32 * 100) / 256).min(99);
					format!("{v:02}")
				})
				.collect::<Vec<String>>()
				.join(" ")
		})
		.collect()
}

/// Converts a grayscale bitmap into rows of ASCII "art," using various Unicode
/// characters to represent ranges of pixel intensity.
///
/// The conversion uses these thresholds (inclusively):
///
/// - `0..=60` -> `"  "` (2 spaces)  
/// - `61..=120` -> `"░░"`  
/// - `121..=180` -> `"▒▒"`  
/// - `181..=240` -> `"▓▓"`  
/// - `241..=255` -> `"█"`  
///
/// # Arguments
/// * `bitmap` - A slice of 8-bit grayscale pixel data.
/// * `width` - The number of pixels per row.
///
/// # Returns
/// A vector of strings, where each string represents one row of the image
/// as a sequence of 2-character intensity symbols.
///
/// # Example
/// ```
/// let bitmap = vec![0, 64, 128, 192, 255];
/// let rows = bitmap_as_ascii_art(&bitmap, 5);
/// assert_eq!(
///     rows,
///     vec![
///         "  ░░▒▒▓▓█"
///     ]
/// );
/// ```
pub fn bitmap_as_ascii_art(bitmap: &[u8], width: usize) -> Vec<String> {
	bitmap
		.chunks(width)
		.map(|row| {
			row.iter()
				.map(|&x| {
					String::from(match x {
						0..=60 => "  ",
						61..=120 => "░░",
						121..=180 => "▒▒",
						181..=240 => "▓▓",
						_ => "█",
					})
				})
				.collect::<Vec<String>>()
				.join("")
		})
		.collect()
}
